#![feature(ptr_metadata)]

use std::any::Any;

use dyncast::{dyncast, DyncastExt};

#[dyncast]
trait Boba<A: 'static> {
    fn supper(&self);
}

struct A;

#[dyncast]
impl Boba<String> for A {
    fn supper(&self) {
        println!("a")
    }
}

#[dyncast]
impl Boba<i32> for A {
    fn supper(&self) {
        println!("i32")
    }
}

struct B;

#[dyncast]
impl Boba<String> for B {
    fn supper(&self) {
        println!("b")
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

    a.dyncast_to::<dyn Boba<String>>().unwrap().supper();
    a.dyncast_to::<dyn Boba<i32>>().unwrap().supper();
    b.dyncast_to::<dyn Boba<String>>().unwrap().supper();

    assert!(a.dyncast_to::<dyn Boba<String>>().is_some());
    assert!(a.dyncast_to::<dyn Boba<usize>>().is_none());
    assert!(a.dyncast_to::<dyn Boba<i32>>().is_some());
    assert!(a.dyncast_to::<dyn Soba>().is_none());

    assert!(b.dyncast_to::<dyn Boba<String>>().is_some());
    assert!(b.dyncast_to::<dyn Boba<Box<usize>>>().is_none());
    assert!(b.dyncast_to::<dyn Soba>().is_some());
}
