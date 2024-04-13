use std::any::Any;

use dyncast::{dyncast, DyncastExt};

#[dyncast]
trait Boba {
    fn supper(&self) -> &'static str;
}

struct A;

#[dyncast]
impl Boba for A {
    fn supper(&self) -> &'static str {
        "a"
    }
}

struct B;

#[dyncast]
impl Boba for B {
    fn supper(&self) -> &'static str {
        "b"
    }
}

#[dyncast]
trait Soba {}

#[dyncast]
impl Soba for B {}

#[test]
fn boba() {
    let a = A;
    let b = B;

    let a = &a as &dyn Any;
    let b = &b as &dyn Any;

    assert!(a.dyncast_to::<dyn Boba>().is_some());
    assert_eq!(a.dyncast_to::<dyn Boba>().unwrap().supper(), "a");

    assert!(b.dyncast_to::<dyn Boba>().is_some());
    assert_eq!(b.dyncast_to::<dyn Boba>().unwrap().supper(), "b");

    assert!(a.dyncast_to::<dyn Soba>().is_none());
    assert!(b.dyncast_to::<dyn Soba>().is_some());
}
