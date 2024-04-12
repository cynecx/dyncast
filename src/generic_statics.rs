//! Specialized form of [generic-statics](https://github.com/cynecx/generic-statics/).

use std::{any::TypeId, marker::PhantomData};

struct Inspect<T>(PhantomData<T>);

impl<T> Inspect<T> {
    const IS_VALID: bool = {
        assert!(std::mem::size_of::<*const ()>() <= 8);
        assert!(std::mem::size_of::<T>() <= 64);
        assert!(std::mem::align_of::<T>() <= 16);
        true
    };
}

/// # Safety
/// `T` must be a bit zeroable type.
#[inline(never)]
#[must_use]
pub unsafe fn generic_static<T: 'static>() -> &'static T {
    assert!(Inspect::<T>::IS_VALID);

    #[allow(unused_assignments)]
    let mut addr: *const () = std::ptr::null();

    // HACK: We have to "use" the generic `T` in some way to force the compiler to emit every
    // instatiation of this function, otherwise rustc might be smart and merge instantiations.
    let type_id = TypeId::of::<T> as *const ();

    #[cfg(all(
        target_arch = "aarch64",
        any(target_os = "macos", target_os = "ios", target_os = "tvos")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "adrp {x}, 1f@PAGE",
            "add {x}, {x}, 1f@PAGEOFF",
            ".pushsection __DATA,__data",
            ".p2align 4, 0",
            "1: .zero 64",
            ".popsection",
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "aarch64",
        any(target_os = "none", target_os = "linux", target_os = "freebsd")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "adrp {x}, 1f",
            "add {x}, {x}, :lo12:1f",
            ".pushsection .bss.generic_statics,\"aw\",@nobits",
            ".p2align 4, 0",
            "1: .zero 64",
            ".popsection",
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_os = "macos", target_os = "ios", target_os = "tvos")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "lea {x}, [rip + 1f]",
            ".pushsection __DATA,__data",
            ".p2align 4, 0",
            "1: .zero 64",
            ".popsection",
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        any(target_os = "none", target_os = "linux", target_os = "freebsd")
    ))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "lea {x}, [rip + 1f]",
            ".pushsection .bss.generic_statics,\"aw\",@nobits",
            ".p2align 4, 0",
            "1: .zero 64",
            ".popsection",
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    unsafe {
        std::arch::asm!(
            "/* {type_id} */",
            "lea {x}, [rip + 1f]",
            ".pushsection .bss.generic_statics,\"bw\"",
            ".p2align 4, 0",
            "1: .zero 64",
            ".popsection",
            type_id = in(reg) type_id,
            x = out(reg) addr,
            options(nostack)
        );
    }

    // In case this is run on targets we don't really support.
    assert!(!addr.is_null());

    unsafe { &*addr.cast::<T>() }
}
