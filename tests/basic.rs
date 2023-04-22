#![feature(ptr_metadata)]

use std::any::Any;

use dyncast::{dyncast, DyncastExt};

#[dyncast]
trait Boba {
    fn supper(&self);
}

struct A;

#[dyncast]
impl Boba for A {
    fn supper(&self) {
        println!("a")
    }
}

struct B;

#[dyncast]
impl Boba for B {
    fn supper(&self) {
        println!("b")
    }
}

#[dyncast]
trait Soba {}

#[test]
fn boba() {
    let a = A;
    let b = B;

    let a = &a as &dyn Any;
    let b = &b as &dyn Any;

    // a.dyncast_to::<dyn Boba>().unwrap().supper();
    // b.dyncast_to::<dyn Boba>().unwrap().supper();

    assert!(a.dyncast_to::<dyn Boba>().is_some());
    assert!(b.dyncast_to::<dyn Boba>().is_some());

    assert!(a.dyncast_to::<dyn Soba>().is_none());
    assert!(b.dyncast_to::<dyn Soba>().is_none());
}
