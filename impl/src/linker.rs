// From the `linkme` crate:
// https://github.com/dtolnay/linkme/blob/b841bae328e844b4ff7f9a8d571df771fbecfc18/impl/src/linker.rs

pub mod elf {
    use std::fmt::Display;

    pub fn section(ident: impl Display, seed: Option<u64>) -> String {
        let seed = seed.map(|seed| format!("{seed}")).unwrap_or_default();
        format!("dyncst_{ident}{seed}")
    }

    pub fn section_start(ident: impl Display, seed: Option<u64>) -> String {
        let seed = seed.map(|seed| format!("{seed}")).unwrap_or_default();
        format!("__start_dyncst_{ident}{seed}")
    }

    pub fn section_stop(ident: impl Display, seed: Option<u64>) -> String {
        let seed = seed.map(|seed| format!("{seed}")).unwrap_or_default();
        format!("__stop_dyncst_{ident}{seed}")
    }
}

pub mod macho {
    use std::{fmt::Display, hash::Hash};

    use crate::hash::hash;

    pub fn section(ident: impl Display + Hash, seed: Option<u64>) -> String {
        format!("__DATA,__dyncst{},regular,no_dead_strip", hash(ident, seed))
    }

    pub fn section_start(ident: impl Display + Hash, seed: Option<u64>) -> String {
        format!("\x01section$start$__DATA$__dyncst{}", hash(ident, seed))
    }

    pub fn section_stop(ident: impl Display + Hash, seed: Option<u64>) -> String {
        format!("\x01section$end$__DATA$__dyncst{}", hash(ident, seed))
    }
}
