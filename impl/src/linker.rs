// From the `linkme` crate:
// https://github.com/dtolnay/linkme/blob/b841bae328e844b4ff7f9a8d571df771fbecfc18/impl/src/linker.rs

use std::hash::Hash;

use proc_macro2::Ident;

#[derive(Debug, Clone, Copy)]
pub struct SectionNameArgs<'a> {
    pub name: &'a Ident,
    pub global_id: &'a str,
    pub seed: Option<u64>,
}

fn symbol_for(section_name_args: SectionNameArgs<'_>) -> crate::hash::Symbol {
    crate::hash::hash(|mut hasher| {
        section_name_args.name.hash(&mut hasher);
        section_name_args.global_id.hash(&mut hasher);
        if let Some(seed) = section_name_args.seed {
            seed.hash(&mut hasher);
        }
    })
}

pub mod elf {
    use crate::linker::{symbol_for, SectionNameArgs};

    pub fn section(descr: SectionNameArgs<'_>) -> String {
        format!("dyncst_{}{}", descr.name, symbol_for(descr))
    }

    pub fn section_start(descr: SectionNameArgs<'_>) -> String {
        format!("__start_dyncst_{}{}", descr.name, symbol_for(descr))
    }

    pub fn section_stop(descr: SectionNameArgs<'_>) -> String {
        format!("__stop_dyncst_{}{}", descr.name, symbol_for(descr))
    }
}

pub mod macho {
    use crate::linker::{symbol_for, SectionNameArgs};

    pub fn section(descr: SectionNameArgs<'_>) -> String {
        format!("__DATA,__dyncst{},regular,no_dead_strip", symbol_for(descr))
    }

    pub fn section_start(descr: SectionNameArgs<'_>) -> String {
        format!("\x01section$start$__DATA$__dyncst{}", symbol_for(descr))
    }

    pub fn section_stop(descr: SectionNameArgs<'_>) -> String {
        format!("\x01section$end$__DATA$__dyncst{}", symbol_for(descr))
    }
}
