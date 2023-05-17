#![feature(ptr_metadata)]

use std::any::Any;

use dyncast::{dyncast, DyncastExt};

#[dyncast(seed = 1337)]
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

#[test]
fn boba() {
    let a = A;
    let a = &a as &dyn Any;
    assert!(a.dyncast_to::<dyn Boba>().is_some());
}
