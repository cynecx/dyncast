/// Basically a reimplementation of std's `Once` but in this case we control the implementation and
/// therefore can make stronger gurantees about the valid memory representation of our `Once`.
/// It is important that our `Once` can be "zero-initialized'.
use std::{
    cell::UnsafeCell,
    panic::{self, AssertUnwindSafe},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    thread::{self, Thread},
    unreachable,
};

use sptr::Strict;
use static_generics::Zeroable;

pub struct Once {
    state: AtomicPtr<Waiter>,
}

unsafe impl Send for Once {}
unsafe impl Sync for Once {}

unsafe impl Zeroable for Once {}

impl Once {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            state: AtomicPtr::new(STATE_INIT_PTR),
        }
    }

    pub fn call_once<R, F: FnOnce() -> R>(&self, f: F) -> Option<R> {
        let packed = Packed::load_acquire(&self.state);
        if packed.is_completed() {
            return None;
        }
        self.call_once_slow(packed, f)
    }

    #[cold]
    fn call_once_slow<R, F: FnOnce() -> R>(&self, packed: Packed, f: F) -> Option<R> {
        let mut packed = packed;

        loop {
            if packed.is_completed() {
                return None;
            }

            if packed.is_init() {
                match self.state.compare_exchange(
                    packed.into_inner(),
                    Packed::new_waiting(None).into_inner(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        let res = panic::catch_unwind(AssertUnwindSafe(|| f()));
                        let final_state = if res.is_ok() {
                            STATE_COMPLETED_PTR
                        } else {
                            STATE_INIT_PTR
                        };

                        let prev = Packed::from_ptr(self.state.swap(final_state, Ordering::AcqRel));
                        assert!(prev.is_waiting());

                        let mut waiter_ptr = prev.waiter();
                        while !waiter_ptr.is_null() {
                            let waiter = unsafe { &*waiter_ptr };

                            let (thread, notified) = {
                                let inner = unsafe { &*waiter.0.get().cast_const() };
                                waiter_ptr = inner.next;
                                (inner.thread.clone(), &inner.notified)
                            };

                            notified.store(true, Ordering::SeqCst);
                            thread.unpark();
                        }

                        match res {
                            Ok(val) => return Some(val),
                            Err(err) => panic::resume_unwind(err),
                        }
                    }
                    Err(prev) => {
                        packed = Packed::from_ptr(prev);
                        continue;
                    }
                }
            }

            if packed.is_waiting() {
                let waiter = Waiter::current_with_next(packed.waiter());

                if let Err(prev) = self.state.compare_exchange(
                    packed.into_inner(),
                    Packed::new_waiting(Some(&waiter as *const _)).into_inner(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    packed = Packed::from_ptr(prev);
                    continue;
                }

                let waiter = unsafe { &*waiter.0.get().cast_const() };

                while !waiter.notified.load(Ordering::SeqCst) {
                    thread::park();
                }

                packed = Packed::load_acquire(&self.state);
                continue;
            }

            unreachable!("invalid state");
        }
    }
}

const STATE_INIT: usize = 0;
const STATE_WAITING: usize = 1;
const STATE_COMPLETED: usize = 2;

const STATE_MASK: usize = 3;
const STATE_WAITER_MASK: usize = !STATE_MASK;

const STATE_INIT_PTR: *mut Waiter = sptr::invalid_mut(STATE_INIT);
const STATE_COMPLETED_PTR: *mut Waiter = sptr::invalid_mut(STATE_COMPLETED);

#[derive(Clone, Copy)]
struct Packed(*mut Waiter);

impl Packed {
    #[inline(always)]
    fn new_waiting(waiter: Option<*const Waiter>) -> Self {
        Self(
            Strict::map_addr(waiter.unwrap_or(sptr::invalid(0)), |addr| {
                (addr & STATE_WAITER_MASK) | STATE_WAITING
            })
            .cast_mut(),
        )
    }

    #[inline(always)]
    const fn from_ptr(ptr: *mut Waiter) -> Self {
        Self(ptr)
    }

    fn load_acquire(state: &AtomicPtr<Waiter>) -> Self {
        Self(state.load(Ordering::Acquire))
    }

    #[inline(always)]
    fn waiter(self) -> *const Waiter {
        Strict::map_addr(self.0.cast_const(), |addr| addr & STATE_WAITER_MASK)
    }

    #[inline(always)]
    fn state(self) -> usize {
        Strict::addr(self.0) & STATE_MASK
    }

    #[inline(always)]
    fn is_waiting(self) -> bool {
        self.state() == STATE_WAITING
    }

    #[inline(always)]
    fn is_init(self) -> bool {
        let res = self.state() == STATE_INIT;
        if res {
            assert!(self.waiter().is_null());
        }
        res
    }

    #[inline(always)]
    fn is_completed(self) -> bool {
        let res = self.state() == STATE_COMPLETED;
        if res {
            assert!(self.waiter().is_null());
        }
        res
    }

    #[inline(always)]
    const fn into_inner(self) -> *mut Waiter {
        self.0
    }
}

#[repr(align(4))]
struct Waiter(UnsafeCell<WaiterInner>);

impl Waiter {
    #[inline(always)]
    fn current_with_next(next: *const Waiter) -> Self {
        Self(UnsafeCell::new(WaiterInner {
            next,
            thread: thread::current(),
            notified: AtomicBool::new(false),
        }))
    }
}

struct WaiterInner {
    next: *const Waiter,
    thread: Thread,
    notified: AtomicBool,
}
