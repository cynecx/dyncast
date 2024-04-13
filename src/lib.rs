//! ## dyncast
//!
//! [![github]](https://github.com/cynecx/dyncast) [![crates-io]](https://crates.io/crates/dyncast)
//!
//! [github]: https://img.shields.io/badge/github-cynecx/dyncast-blue?logo=github
//! [crates-io]: https://img.shields.io/crates/v/dyncast.svg?logo=rust
//!
//! **Fair warning**: This crate is a **proof of concept**. The soundness of this crate has **not** been validated. **Use at your own risk**.
//!
//! This library provides opt-in type downcasting to `dyn Trait`s.
//!
//! The entrypoint for this library is the [`dyncast`] proc-macro, which should be applied on every `trait` and `impl`, you'd want to enable downcasting to. The proc-macro will additionally implement [`Dyncast`] for the selected trait (`dyn Trait`), which can be used to check whether a concrete type implements such trait.
//!
//! ### Example
//!
//! ```rust
//! use std::any::Any;
//!
//! use dyncast::{dyncast, DyncastExt};
//!
//! #[dyncast]
//! trait Foo {
//!     fn bar(&self);
//! }
//!
//! #[dyncast]
//! impl Foo for () {
//!     fn bar(&self) {
//!         println!("a")
//!     }
//! }
//!
//! #[test]
//! fn boba() {
//!     let a = &() as &dyn Any;
//!     assert!(a.dyncast_to::<dyn Foo>().is_some());
//! }
//! ```
use std::any::Any;

/// [This](`dyncast`) proc-macro can be used on trait definitions and trait impls.
///
/// ```
/// use dyncast::dyncast;
///
/// #[dyncast]
/// trait Foo {}
///
/// #[dyncast]
/// impl Foo for () {}
///
/// # fn main() {}
/// ```
///
/// [`dyncast`] also supports traits with generics. However, this is limited type parameters.
///
/// ```
/// use dyncast::dyncast;
///
/// #[dyncast]
/// trait Foo<T: 'static> {}
///
/// #[dyncast]
/// impl Foo<String> for () {}
///
/// # fn main() {}
/// ```
pub use dyncast_impl::dyncast;

#[doc(hidden)]
pub mod private;

mod generic_statics;
mod global;
mod map;
mod once;
mod ptr;

pub trait Dyncast: Any {
    fn dyncast_from<T: ?Sized + Any>(source: &T) -> Option<&Self>;
}

/// Provides a shorthand method [`dyncast_to`](`DyncastExt::dyncast_to`).
///
/// ```
/// use dyncast::{dyncast, DyncastExt};
///
/// #[dyncast]
/// trait Bar {}
///
/// fn foo(val: &dyn std::any::Any) {
///     assert!(val.dyncast_to::<dyn Bar>().is_none());
/// }
///
/// # fn main() {}
/// ```
pub trait DyncastExt {
    fn dyncast_to<T: ?Sized + Dyncast>(&self) -> Option<&T>;
}

impl<T: ?Sized + Any> DyncastExt for T {
    #[inline(always)]
    fn dyncast_to<D: ?Sized + Dyncast>(&self) -> Option<&D> {
        D::dyncast_from(self)
    }
}
