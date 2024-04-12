use std::any::Any;

use dyncast::DyncastExt;

mod boba {
    use dyncast::dyncast;

    #[dyncast(global_id = "dup::ja")]
    pub trait Foo {}
}

mod soba {
    use dyncast::dyncast;

    #[dyncast(global_id = "dup::ja")]
    pub trait Foo {}
}

#[test]
#[should_panic]
fn check_collision_first() {
    let obj = &() as &dyn Any;
    obj.dyncast_to::<dyn boba::Foo>();
}

#[test]
#[should_panic]
fn check_collision_second() {
    let obj = &() as &dyn Any;
    obj.dyncast_to::<dyn soba::Foo>();
}
