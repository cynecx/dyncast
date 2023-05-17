pub use std::any::TypeId;

pub use crate::map::LazyTypeMap;
pub use crate::Dyncast;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub self_type_id: TypeId,
    pub generics_type_id: Option<TypeId>,
}

#[derive(Copy, Clone)]
pub struct Descriptor {
    pub(crate) self_type_id: TypeId,
    pub(crate) generics_type_id: Option<TypeId>,
    attach_vtable: *const (),
}

unsafe impl Send for Descriptor {}
unsafe impl Sync for Descriptor {}

impl Descriptor {
    #[inline]
    pub unsafe fn new<T: ?Sized>(
        self_type_id: TypeId,
        attach_vtable: unsafe fn(*const ()) -> *const T,
    ) -> Self {
        Self {
            self_type_id,
            generics_type_id: None,
            attach_vtable: attach_vtable as *const (),
        }
    }

    #[inline]
    pub unsafe fn new_generics<T: ?Sized>(
        self_type_id: TypeId,
        generics_type_id: TypeId,
        attach_vtable: unsafe fn(*const ()) -> *const T,
    ) -> Self {
        Self {
            self_type_id,
            generics_type_id: Some(generics_type_id),
            attach_vtable: attach_vtable as *const (),
        }
    }

    #[inline]
    pub unsafe fn attach_vtable<T: ?Sized>(&self) -> unsafe fn(*const ()) -> *const T {
        std::mem::transmute(self.attach_vtable)
    }
}
