use std::any::Any;

/// [This](`dyncast`) proc-macro can be used on trait definitions and trait impls.
///
/// ```
/// use dyncast::dyncast;
///
/// #[dyncast(global_id = "test1::Foo")]
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
/// #[dyncast(global_id = "test2::Foo")]
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
mod map;
mod once;
mod ptr;

pub trait Dyncast: Any {
    fn dyncast_from<T: ?Sized + Any>(source: &T) -> Option<&Self>;
}

/// Provides a shorthand method [dyncast_to](`DyncastExt::dyncast_to`).
///
/// ```
/// use dyncast::{dyncast, DyncastExt};
///
/// #[dyncast(global_id = "universe::Bar")]
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
