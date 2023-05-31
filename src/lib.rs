#![feature(ptr_metadata)]

use std::any::Any;

pub use dyncast_impl::dyncast;

#[doc(hidden)]
pub mod private;

mod map;
mod once;

pub trait Dyncast: Any {
    fn dyncast_from<T: ?Sized + Any>(source: &T) -> Option<&Self>;
}

pub trait DyncastExt {
    fn dyncast_to<T: ?Sized + Dyncast>(&self) -> Option<&T>;
}

impl<T: ?Sized + Any> DyncastExt for T {
    #[inline]
    fn dyncast_to<D: ?Sized + Dyncast>(&self) -> Option<&D> {
        D::dyncast_from(self)
    }
}
