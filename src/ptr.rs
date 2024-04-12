use std::marker::PhantomData;

struct Inspect<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Inspect<T> {
    const IS_DYN_TRAIT: bool = {
        assert!(std::mem::size_of::<*const T>() == std::mem::size_of::<PtrComponents>());
        assert!(std::mem::align_of::<*const T>() == std::mem::align_of::<PtrComponents>());
        true
    };
}

#[repr(C)]
union PtrRepr<T: ?Sized> {
    const_ptr: *const T,
    mut_ptr: *mut T,
    components: PtrComponents,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct PtrComponents {
    data_pointer: *const (),
    vtable: *const (),
}

/// # Safety
///
/// `T` must be a dyn trait.
#[inline(always)]
pub unsafe fn metadata<T: ?Sized>(val: *const T) -> *const () {
    assert!(Inspect::<T>::IS_DYN_TRAIT);
    unsafe { PtrRepr { const_ptr: val }.components.vtable }
}

/// # Safety
///
/// `T` must be a dyn trait.
#[inline(always)]
pub unsafe fn from_raw_parts<T: ?Sized>(data_pointer: *const (), vtable: *const ()) -> *const T {
    assert!(Inspect::<T>::IS_DYN_TRAIT);
    unsafe {
        PtrRepr {
            components: PtrComponents {
                data_pointer,
                vtable,
            },
        }
        .const_ptr
    }
}
