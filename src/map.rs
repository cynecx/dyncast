use std::{
    any::TypeId,
    cell::UnsafeCell,
    collections::HashMap,
    iter,
    mem::{self, MaybeUninit},
    ptr,
};

use static_generics::Zeroable;

use crate::{once::Once, private::Descriptor};

type Inner<T> = HashMap<TypeId, Descriptor<T>>;

pub struct LazyTypeMap<T: ?Sized> {
    once: Once,
    inner: UnsafeCell<MaybeUninit<Inner<T>>>,
}

// SAFETY: All fields implement `Zeroable`.
unsafe impl<T: ?Sized> Zeroable for LazyTypeMap<T> {}

impl<T: ?Sized> LazyTypeMap<T> {
    pub fn current() -> &'static Self {
        static_generics::static_generic()
    }

    pub unsafe fn get_or_init(&self, start: *const *const (), end: *const *const ()) -> &Inner<T> {
        let inner = self.inner.get();
        self.once.call_once(|| unsafe {
            let map = descriptors(start, end)
                .map(|entry| {
                    let descriptor = (entry)();
                    (descriptor.ty_id(), descriptor)
                })
                .collect();
            ptr::write(inner, MaybeUninit::new(map));
        });
        unsafe { (&*self.inner.get().cast_const()).assume_init_ref() }
    }
}

unsafe fn descriptors<T: ?Sized>(
    mut start: *const *const (),
    end: *const *const (),
) -> impl Iterator<Item = fn() -> Descriptor<T>> {
    iter::from_fn(move || loop {
        if start == end {
            return None;
        }
        let entry = unsafe { *start };
        start = start.add(1);
        if entry.is_null() {
            continue;
        }
        return Some(unsafe { mem::transmute::<*const (), fn() -> Descriptor<T>>(entry) });
    })
}
