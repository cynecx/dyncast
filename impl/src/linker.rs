// From the `linkme` crate:
// https://github.com/dtolnay/linkme/blob/b841bae328e844b4ff7f9a8d571df771fbecfc18/impl/src/linker.rs

pub mod linux {
    use std::fmt::Display;

    pub fn section(ident: impl Display) -> String {
        format!("dyncst_{}", ident)
    }

    pub fn section_start(ident: impl Display) -> String {
        format!("__start_dyncst_{}", ident)
    }

    pub fn section_stop(ident: impl Display) -> String {
        format!("__stop_dyncst_{}", ident)
    }
}

pub mod macho {
    use std::{fmt::Display, hash::Hash};

    use crate::hash::hash;

    pub fn section(ident: impl Display + Hash) -> String {
        format!("__DATA,__dyncst{},regular,no_dead_strip", hash(ident))
    }

    pub fn section_start(ident: impl Display + Hash) -> String {
        format!("\x01section$start$__DATA$__dyncst{}", hash(ident))
    }

    pub fn section_stop(ident: impl Display + Hash) -> String {
        format!("\x01section$end$__DATA$__dyncst{}", hash(ident))
    }
}
