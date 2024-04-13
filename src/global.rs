use std::{any::TypeId, collections::HashMap, hash::Hash, mem, sync::OnceLock};

use crate::private::{Descriptor, Entry, PartialDescriptor};

#[cfg(any(
    target_os = "none",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
))]
extern "Rust" {
    #[cfg_attr(
        any(target_os = "none", target_os = "linux", target_os = "freebsd"),
        link_name = "__start_dyncst_entries"
    )]
    #[cfg_attr(
        any(target_os = "macos", target_os = "ios", target_os = "tvos"),
        link_name = "\x01section$start$__DATA$__dyncst_entries"
    )]
    static DYNCAST_START: Entry;

    #[cfg_attr(
        any(target_os = "none", target_os = "linux", target_os = "freebsd"),
        link_name = "__stop_dyncst_entries"
    )]
    #[cfg_attr(
        any(target_os = "macos", target_os = "ios", target_os = "tvos"),
        link_name = "\x01section$end$__DATA$__dyncst_entries"
    )]
    static DYNCAST_STOP: Entry;
}

#[cfg(target_os = "windows")]
#[link_section = ".dyncst_entries$a"]
static DYNCAST_START: [Entry; 0] = [];

#[cfg(target_os = "windows")]
#[link_section = ".dyncst_entries$c"]
static DYNCAST_STOP: [Entry; 0] = [];

#[cfg(not(any(
    target_os = "none",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "windows",
)))]
std::compile_error!("dyncast is not supported on this platform");

pub type DynTraitTypeId = TypeId;

pub type SelfTypeId = TypeId;

pub struct Global {
    pub dyn_trait_map: HashMap<DynTraitTypeId, HashMap<SelfTypeId, PartialDescriptor>>,
}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl Global {
    pub fn singleton() -> &'static Global {
        static INIT: OnceLock<Global> = OnceLock::new();
        INIT.get_or_init(Self::build)
    }

    fn build() -> Self {
        let mut descriptors = unsafe {
            descriptors(
                std::ptr::addr_of!(DYNCAST_START) as *const Entry,
                std::ptr::addr_of!(DYNCAST_STOP) as *const Entry,
            )
            .collect::<Vec<_>>()
        };

        descriptors.sort_unstable_by_key(|descriptor| descriptor.dyn_trait_id);

        let dyn_trait_map: HashMap<DynTraitTypeId, HashMap<SelfTypeId, PartialDescriptor>> =
            group_and_collect(
                descriptors.iter().copied(),
                |descriptor| descriptor.dyn_trait_id,
                |descriptor| {
                    (
                        descriptor.self_type_id,
                        PartialDescriptor {
                            attach_vtable_fn: descriptor.attach_vtable_fn,
                        },
                    )
                },
            );

        Self { dyn_trait_map }
    }
}

unsafe fn descriptors(start: *const Entry, end: *const Entry) -> impl Iterator<Item = Descriptor> {
    assert!(start <= end);

    let mut curr = start;

    std::iter::from_fn(move || {
        if curr == end {
            return None;
        }

        let entry = unsafe { &*curr };
        curr = curr.add(1);

        let entry_ptr = entry.0.get().cast_const();
        assert!(!entry_ptr.is_null());

        let descriptor = unsafe { (*entry_ptr)() };
        Some(descriptor)
    })
}

fn group_and_collect<T, K, E, C>(
    iter: impl Iterator<Item = T>,
    group_key_fn: impl Fn(&T) -> K,
    entry_fn: impl Fn(T) -> E,
) -> HashMap<K, C>
where
    K: Eq + Hash + Copy,
    C: Default + Extend<E>,
{
    let mut map: HashMap<K, C> = HashMap::new();
    let mut curr_group: Option<(K, C)> = None;

    for item in iter {
        let group_key = group_key_fn(&item);

        let (curr_group_key, curr_group_entries) =
            curr_group.get_or_insert_with(|| (group_key, C::default()));

        if *curr_group_key != group_key {
            let previous_group_key = mem::replace(curr_group_key, group_key);
            let previous_group_entries = mem::take(curr_group_entries);
            map.insert(previous_group_key, previous_group_entries);
        }

        curr_group_entries.extend(Some(entry_fn(item)));
    }

    if let Some((last_group_key, last_group_entries)) = curr_group {
        map.insert(last_group_key, last_group_entries);
    }

    map
}
