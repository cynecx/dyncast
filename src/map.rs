use std::{
    any::Any,
    cell::UnsafeCell,
    collections::HashMap,
    iter,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

use crate::{
    once::Once,
    private::{Descriptor, Key, PartialDescriptor},
};

type Map = HashMap<Key, PartialDescriptor>;

pub struct LazyTypeMap<T: ?Sized> {
    once: Once,
    inner: UnsafeCell<MaybeUninit<Map>>,
    _tag: PhantomData<T>,
}

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

impl<T: ?Sized + Any> LazyTypeMap<T> {
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            _tag: PhantomData,
        }
    }

    /// # Safety
    /// This is an internal api used by the proc-macro generated code.
    pub unsafe fn current() -> &'static Self {
        unsafe { crate::generic_statics::generic_static() }
    }

    /// # Safety
    /// This is an internal api used by the proc-macro generated code.
    pub unsafe fn get_or_init(
        &self,
        start: *const *const (),
        end: *const *const (),
    ) -> InitializedTypeMap<'_> {
        let inner = self.inner.get();

        self.once.call_once(|| {
            let descriptors = unsafe { descriptors(start, end) };

            let mut has_seen_null = false;

            let map = descriptors
                .filter_map(|entry| match entry {
                    Some(entry) => Some(entry),
                    None if !has_seen_null => {
                        has_seen_null = true;
                        None
                    }
                    None => panic_on_namespace_collision::<T>(),
                })
                .map(|entry| {
                    let descriptor = (entry)();
                    let key = Key {
                        self_type_id: descriptor.self_type_id,
                        generics_type_id: descriptor.generics_type_id,
                    };
                    let val = PartialDescriptor {
                        attach_vtable_fn: descriptor.attach_vtable_fn,
                    };
                    (key, val)
                })
                .collect();

            ptr::write(inner, MaybeUninit::new(map));
        });

        let map = unsafe { (*self.inner.get().cast_const()).assume_init_ref() };
        InitializedTypeMap(map)
    }
}

impl<T: ?Sized + Any> Default for LazyTypeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InitializedTypeMap<'a>(&'a Map);

impl<'a> InitializedTypeMap<'a> {
    #[inline]
    pub fn get(&self, key: &Key) -> Option<&PartialDescriptor> {
        self.0.get(key)
    }
}

unsafe fn descriptors(
    start: *const *const (),
    end: *const *const (),
) -> impl Iterator<Item = Option<fn() -> Descriptor>> {
    assert!(start <= end);

    let mut curr = start;

    iter::from_fn(move || {
        if curr == end {
            return None;
        }

        let entry = unsafe { *curr };
        curr = curr.add(1);

        let item = if entry.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute::<*const (), fn() -> Descriptor>(entry) })
        };

        Some(item)
    })
}

#[track_caller]
fn panic_on_namespace_collision<T: ?Sized + Any>() -> ! {
    panic!(
        "dyncast hash collision detected. A global_id adjustment might be required for this dyncast trait ({}).",
        std::any::type_name::<T>()
    )
}
