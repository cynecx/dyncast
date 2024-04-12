pub use std::any::TypeId;
use std::cell::UnsafeCell;

pub use crate::map::LazyTypeMap;
pub use crate::Dyncast;

pub mod ptr {
    pub use crate::ptr::*;
}

pub type Entry = SyncUnsafeCell<unsafe fn() -> Descriptor>;

#[repr(transparent)]
pub struct SyncUnitPtr(*const ());

unsafe impl Send for SyncUnitPtr {}
unsafe impl Sync for SyncUnitPtr {}

#[repr(transparent)]
pub struct SyncUnsafeCell<T>(pub UnsafeCell<T>);

impl<T> SyncUnsafeCell<T> {
    #[inline]
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }
}

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

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
    pub(crate) dyn_trait_id: TypeId,
    pub(crate) attach_vtable_fn: *const (),
}

unsafe impl Send for Descriptor {}
unsafe impl Sync for Descriptor {}

impl Descriptor {
    #[inline]
    pub unsafe fn new<T: ?Sized>(
        self_type_id: TypeId,
        dyn_trait_id: TypeId,
        attach_vtable_fn: unsafe fn(*const ()) -> *const T,
    ) -> Self {
        Self {
            self_type_id,
            dyn_trait_id,
            attach_vtable_fn: attach_vtable_fn as *const (),
        }
    }
}
