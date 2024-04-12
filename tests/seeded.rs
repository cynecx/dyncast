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

#[test]
fn boba() {
    let a = A;
    let a = &a as &dyn Any;
    assert!(a.dyncast_to::<dyn Boba>().is_some());
    assert_eq!(a.dyncast_to::<dyn Boba>().unwrap().supper(), "a");
}
