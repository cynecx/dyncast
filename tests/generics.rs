use std::any::Any;

use dyncast::{dyncast, DyncastExt};

#[dyncast(global_id = "generics::Boba")]
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

#[dyncast(global_id = "generics::Soba")]
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

#[dyncast(global_id = "generics::Convert")]
trait Convert<To> {
    fn convert_to(&self) -> To;
}

struct Conv(usize);

#[dyncast]
impl Convert<String> for Conv {
    fn convert_to(&self) -> String {
        format!("{}", self.0)
    }
}

#[dyncast]
impl Convert<usize> for Conv {
    fn convert_to(&self) -> usize {
        self.0
    }
}

#[test]
fn convert() {
    let p = Box::new(Conv(1337)) as Box<dyn Any>;
    let p = &*p;
    assert_eq!(
        p.dyncast_to::<dyn Convert<usize>>().unwrap().convert_to(),
        1337
    );
    assert_eq!(
        p.dyncast_to::<dyn Convert<String>>().unwrap().convert_to(),
        "1337"
    );
}
