#![allow(dead_code)]

// From the `linkme` crate:
// https://github.com/dtolnay/linkme/blob/b841bae328e844b4ff7f9a8d571df771fbecfc18/impl/src/linker.rs

// Note: Keep this in sync with `src/global.rs`.

pub mod elf {
    pub const SECTION: &str = "dyncst_entries";
    pub const SECTION_START: &str = "__start_dyncst_entries";
    pub const SECTION_STOP: &str = "__stop_dyncst_entries";
}

pub mod macho {
    pub const SECTION: &str = "__DATA,__dyncst_entries,regular,no_dead_strip";
    pub const SECTION_START: &str = "\x01section$start$__DATA$__dyncst_entries";
    pub const SECTION_STOP: &str = "\x01section$end$__DATA$__dyncst_entries";
}

pub mod windows {
    pub const SECTION: &str = ".dyncst_entries$b";
    pub const SECTION_START: &str = ".dyncst_entries$a";
    pub const SECTION_STOP: &str = ".dyncst_entries$c";
}
