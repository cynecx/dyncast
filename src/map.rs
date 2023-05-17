use std::{
    cell::UnsafeCell,
    collections::HashMap,
    iter,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

use static_generics::Zeroable;

use crate::{
    once::Once,
    private::{Descriptor, Key},
};

type Map = HashMap<Key, Descriptor>;

pub struct LazyTypeMap<T: ?Sized> {
    once: Once,
    inner: UnsafeCell<MaybeUninit<Map>>,
    _tag: PhantomData<T>,
}

// SAFETY: All fields implement `Zeroable`.
unsafe impl<T: ?Sized> Zeroable for LazyTypeMap<T> {}

// SAFETY: All non-zst fields implement `Send`.
unsafe impl<T: ?Sized> Send for LazyTypeMap<T>
where
    Once: Send,
    Map: Send,
{
}

// SAFETY: All non-zst fields implement `Sync`.
unsafe impl<T: ?Sized> Sync for LazyTypeMap<T>
where
    Once: Sync,
    Map: Sync,
{
}

impl<T: ?Sized> LazyTypeMap<T> {
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            _tag: PhantomData,
        }
    }

    pub fn current() -> &'static Self {
        static_generics::static_generic()
    }

    /// # Safety
    /// This is an internal api used by the proc-macro generated code.
    pub unsafe fn get_or_init(&self, start: *const *const (), end: *const *const ()) -> &Map {
        let inner = self.inner.get();
        self.once.call_once(|| unsafe {
            validate_descriptors(start, end);
            let map = descriptors(start, end)
                .map(|entry| {
                    let descriptor = (entry)();
                    (
                        Key {
                            self_type_id: descriptor.self_type_id,
                            generics_type_id: descriptor.generics_type_id,
                        },
                        descriptor,
                    )
                })
                .collect();
            ptr::write(inner, MaybeUninit::new(map));
        });
        unsafe { (*self.inner.get().cast_const()).assume_init_ref() }
    }
}

unsafe fn descriptors(
    mut start: *const *const (),
    end: *const *const (),
) -> impl Iterator<Item = fn() -> Descriptor> {
    iter::from_fn(move || loop {
        if start == end {
            return None;
        }
        let entry = unsafe { *start };
        start = start.add(1);
        if entry.is_null() {
            continue;
        }
        return Some(unsafe { mem::transmute::<*const (), fn() -> Descriptor>(entry) });
    })
}

unsafe fn validate_descriptors(mut start: *const *const (), end: *const *const ()) {
    let mut has_null = false;
    while start != end {
        let entry = unsafe { *start };
        if entry.is_null() {
            if has_null {
                panic!("dyncast hash collision detected. Specify a custom seed for this dyncast trait.")
            } else {
                has_null = true;
            }
        }
        start = start.add(1);
    }
}
