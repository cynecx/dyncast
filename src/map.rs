use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr,
};

use crate::{global::Global, once::Once, private::PartialDescriptor};

type MapRef = &'static HashMap<TypeId, PartialDescriptor>;
type Inner = Option<MapRef>;

pub struct LazyTypeMap<T: ?Sized> {
    once: Once,
    inner: UnsafeCell<MaybeUninit<Inner>>,
    _tag: PhantomData<T>,
}

// SAFETY: All non-zst fields implement `Send`.
unsafe impl<T: ?Sized> Send for LazyTypeMap<T>
where
    Once: Send,
    Inner: Send,
{
}

// SAFETY: All non-zst fields implement `Sync`.
unsafe impl<T: ?Sized> Sync for LazyTypeMap<T>
where
    Once: Sync,
    Inner: Sync,
{
}

impl<T: ?Sized + Any> LazyTypeMap<T> {
    const fn new() -> Self {
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
    pub unsafe fn get_or_init(&self) -> InitializedTypeMap<'_> {
        let inner = self.inner.get();

        self.once.call_once(|| {
            let global = Global::singleton();
            let map = global.dyn_trait_map.get(&TypeId::of::<T>());
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

pub struct InitializedTypeMap<'a>(&'a Option<MapRef>);

impl<'a> InitializedTypeMap<'a> {
    #[inline]
    pub fn get(&self, self_type_id: TypeId) -> Option<&PartialDescriptor> {
        self.0.as_ref()?.get(&self_type_id)
    }
}
