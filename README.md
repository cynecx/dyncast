## dyncast

[<img alt="github" src="https://img.shields.io/badge/github-cynecx/dyncast-blue?logo=github">](https://github.com/cynecx/dyncast)
[<img alt="crates.io" src="https://img.shields.io/crates/v/dyncast.svg?logo=rust">](https://crates.io/crates/dyncast)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-dyncast-66c2a5?labelColor=555555&logo=docs.rs">](https://docs.rs/dyncast)

Proof of concept.

This library provides opt-in type downcasting to `dyn Trait`s.

**Fair warning**: The soundness of this approach has not been validated.

```rust
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

    assert!(a.dyncast_to::<dyn Boba>().is_some());
    assert!(b.dyncast_to::<dyn Boba>().is_some());

    assert!(a.dyncast_to::<dyn Soba>().is_none());
    assert!(b.dyncast_to::<dyn Soba>().is_none());
}
```

### Platform support

This crate has been tested and validated on the following platforms:

- macOS `x86_64`, `aarch64`
- Linux `x86_64`, `aarch64`
- Windows 1X `x86_64`
