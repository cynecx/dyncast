use std::{any::TypeId, collections::HashMap, iter, mem};

use once_cell::sync::Lazy;

pub use crate::Dyncast;

pub type TypeMap<T> = Lazy<HashMap<TypeId, Descriptor<T>>>;

#[derive(Copy, Clone)]
pub struct Descriptor<T: ?Sized> {
    type_id: TypeId,
    attach_vtable: unsafe fn(*const ()) -> *const T,
}

impl<T: ?Sized> Descriptor<T> {
    #[inline]
    pub unsafe fn new(type_id: TypeId, attach_vtable: unsafe fn(*const ()) -> *const T) -> Self {
        Self {
            type_id,
            attach_vtable,
        }
    }

    #[inline]
    pub fn ty_id(&self) -> TypeId {
        self.type_id
    }

    #[inline]
    pub fn attach_vtable(&self) -> unsafe fn(*const ()) -> *const T {
        self.attach_vtable
    }
}

pub unsafe fn descriptors<T: ?Sized>(
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
