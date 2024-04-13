use std::any::Any;

use dyncast::DyncastExt;

mod boba {
    use dyncast::dyncast;

    #[dyncast]
    pub trait Foo {
        fn id(&self) -> &'static str {
            "boba::Foo"
        }
    }

    #[dyncast]
    impl Foo for () {}
}

mod soba {
    use dyncast::dyncast;

    #[dyncast]
    pub trait Foo {
        fn id(&self) -> &'static str {
            "soba::Foo"
        }
    }

    #[dyncast]
    impl Foo for () {}
}

#[test]
fn check_trait_with_same_name() {
    let obj = &() as &dyn Any;
    assert_eq!(obj.dyncast_to::<dyn boba::Foo>().unwrap().id(), "boba::Foo");
    assert_eq!(obj.dyncast_to::<dyn soba::Foo>().unwrap().id(), "soba::Foo");
}
