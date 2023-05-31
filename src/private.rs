use std::any::TypeId;

pub use crate::map::LazyTypeMap;
pub use crate::Dyncast;

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
