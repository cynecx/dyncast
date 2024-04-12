pub use std::any::TypeId;

pub use crate::map::LazyTypeMap;
pub use crate::Dyncast;

pub mod ptr {
    pub use crate::ptr::*;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub self_type_id: TypeId,
    pub generics_type_id: Option<TypeId>,
}

#[derive(Copy, Clone)]
pub struct PartialDescriptor {
    pub(crate) attach_vtable_fn: *const (),
}

impl PartialDescriptor {
    #[inline]
    pub unsafe fn attach_vtable_fn<T: ?Sized>(&self) -> unsafe fn(*const ()) -> *const T {
        std::mem::transmute(self.attach_vtable_fn)
    }
}

unsafe impl Send for PartialDescriptor {}
unsafe impl Sync for PartialDescriptor {}

#[derive(Copy, Clone)]
pub struct Descriptor {
    pub(crate) self_type_id: TypeId,
    pub(crate) generics_type_id: Option<TypeId>,
    pub(crate) attach_vtable_fn: *const (),
}

impl Descriptor {
    #[inline]
    pub unsafe fn new<T: ?Sized>(
        self_type_id: TypeId,
        attach_vtable_fn: unsafe fn(*const ()) -> *const T,
    ) -> Self {
        Self {
            self_type_id,
            generics_type_id: None,
            attach_vtable_fn: attach_vtable_fn as *const (),
        }
    }

    #[inline]
    pub unsafe fn new_generics<T: ?Sized>(
        self_type_id: TypeId,
        generics_type_id: TypeId,
        attach_vtable_fn: unsafe fn(*const ()) -> *const T,
    ) -> Self {
        Self {
            self_type_id,
            generics_type_id: Some(generics_type_id),
            attach_vtable_fn: attach_vtable_fn as *const (),
        }
    }
}
