/*!
This example shows how to create a custom cache validator and use it to cache virtual memory reads on a process.
It also provides an example on how to interact with a cache externally and invalidating values quickly.

The example simply reads the header of the provided process twice.

# Usage:
Open process and load the given module with the default dtb.
```bash
cargo run --release --example open_process -- -vvv -c kvm --os win32 --process explorer.exe -m KERNEL32.DLL
```

Overwrite dtb with a custom one:
```bash
cargo run --release --example cached_view -- -vv -c kvm --os win32 --process explorer.exe -m KERNEL32.DLL
```
*/
use ::std::sync::atomic::Ordering;
use std::sync::{
    atomic::{AtomicI32, AtomicU8},
    Arc,
};

use clap::*;
use log::{info, Level};

use memflow::prelude::v1::*;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InvalidationFlags {
    Always,
    Tick,
}

struct ExternallyControlledValidator {
    validator_next_flags: Arc<AtomicU8>,
    validator_tick_count: Arc<AtomicI32>,
}

impl ExternallyControlledValidator {
    pub fn new() -> Self {
        Self {
            validator_next_flags: Arc::new(AtomicU8::new(InvalidationFlags::Always as u8)),
            validator_tick_count: Arc::new(AtomicI32::new(0)),
        }
    }

    pub fn set_next_flags(&mut self, flags: InvalidationFlags) {
        self.validator_next_flags
            .store(flags as u8, Ordering::SeqCst);
    }

    pub fn set_tick_count(&mut self, tick_count: i32) {
        self.validator_tick_count
            .store(tick_count, Ordering::SeqCst);
    }

    pub fn validator(&self) -> CustomValidator {
        CustomValidator::new(
            self.validator_next_flags.clone(),
            self.validator_tick_count.clone(),
        )
    }
}

#[derive(Copy, Clone)]
struct ValidatorSlot {
    value: i32,
    flags: InvalidationFlags,
}

#[derive(Clone)]
pub struct CustomValidator {
    slots: Vec<ValidatorSlot>,

    // The invalidation flags used for the next read or write.
    next_flags: Arc<AtomicU8>,
    next_flags_local: InvalidationFlags,

    // last_count is used to quickly invalidate slots without having to
    // iterate over all slots and invalidating manually.
    last_count: i32,

    // frame count is the externally controlled frame number that will
    // invalidate specific caches when it is increased.
    tick_count: Arc<AtomicI32>,
    tick_count_local: i32,
}

impl CustomValidator {
    pub fn new(next_flags: Arc<AtomicU8>, tick_count: Arc<AtomicI32>) -> Self {
        Self {
            slots: vec![],

            next_flags,
            next_flags_local: InvalidationFlags::Always,

            last_count: 0,

            tick_count,
            tick_count_local: -1,
        }
    }
}

impl CacheValidator for CustomValidator {
    // Create a vector containing all slots with a predefined invalid state.
    fn allocate_slots(&mut self, slot_count: usize) {
        self.slots.resize(
            slot_count,
            ValidatorSlot {
                value: -1,
                flags: InvalidationFlags::Always,
            },
        );
    }

    // This function is invoked on every batch of memory operations.
    // This simply updates the internal state and reads the Atomic variables for the upcoming validations.
    fn update_validity(&mut self) {
        self.last_count = self.last_count.wrapping_add(1);

        // SAFETY: next_flags is guaranteed to be of type InvalidationFlags
        self.next_flags_local = unsafe {
            std::mem::transmute::<_, InvalidationFlags>(self.next_flags.load(Ordering::SeqCst))
        };

        self.tick_count_local = self.tick_count.load(Ordering::SeqCst);
    }

    // This simply returns true or false if the slot is valid or not.
    // `last_count` is used here to invalidate slots quickly without requiring to iterate over the entire slot list.
    fn is_slot_valid(&self, slot_id: usize) -> bool {
        // in case we read / write the same page with different flags we force invalidate this slot instantly
        if self.next_flags_local != self.slots[slot_id].flags {
            return false;
        }

        match self.slots[slot_id].flags {
            InvalidationFlags::Always => self.slots[slot_id].value == self.last_count,
            InvalidationFlags::Tick => self.slots[slot_id].value == self.tick_count_local,
        }
    }

    // In case the cache is being updates this function marks the slot as being valid.
    fn validate_slot(&mut self, slot_id: usize) {
        match self.next_flags_local {
            InvalidationFlags::Always => self.slots[slot_id].value = self.last_count,
            InvalidationFlags::Tick => self.slots[slot_id].value = self.tick_count_local,
        }

        self.slots[slot_id].flags = self.next_flags_local;
    }

    // In case a slot has to be freed this function resets it to the default values.
    fn invalidate_slot(&mut self, slot_id: usize) {
        self.slots[slot_id].value = -1;
        self.slots[slot_id].flags = InvalidationFlags::Always;
    }
}

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, module_name) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    let mut process = os
        .into_process_by_name(proc_name)
        .expect("unable to find process");
    println!("{:?}", process.info());

    // retrieve module info
    let module_info = process
        .module_by_name(module_name)
        .expect("unable to find module in process");
    println!("{module_info:?}");

    // create the validator
    let mut validator_controller = ExternallyControlledValidator::new();
    let validator = validator_controller.validator();

    // create CachedView over the processes MemoryView.
    let proc_arch = process.info().proc_arch;
    let mut cached_process = CachedView::builder(process)
        .arch(proc_arch)
        .validator(validator)
        .cache_size(size::mb(10))
        .build()
        .expect("unable to build cache for process");

    // set the next read to be invalidated only by tick changes
    validator_controller.set_next_flags(InvalidationFlags::Tick);
    info!("reading module_info.base");
    let _header: [u8; 0x1000] = cached_process
        .read(module_info.base)
        .data_part()
        .expect("unable to read pe header");

    info!("reading module_info.base from cache");
    let _header: [u8; 0x1000] = cached_process
        .read(module_info.base)
        .data_part()
        .expect("unable to read pe header");

    // change the frame number to invalidate the cache
    validator_controller.set_tick_count(1);

    // read again with the invalidation flags still in place
    info!("reading module_info.base again with invalid cache");
    let _header: [u8; 0x1000] = cached_process
        .read(module_info.base)
        .data_part()
        .expect("unable to read pe header");

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("open_process example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("os")
                .long("os")
                .short('o')
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("process")
                .long("process")
                .short('p')
                .action(ArgAction::Set)
                .required(true)
                .default_value("explorer.exe"),
        )
        .arg(
            Arg::new("module")
                .long("module")
                .short('m')
                .action(ArgAction::Set)
                .required(true)
                .default_value("KERNEL32.DLL"),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, &str)> {
    let log_level = match matches.get_count("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simplelog::TermLogger::init(
        log_level.to_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let conn_iter = matches
        .indices_of("connector")
        .zip(matches.get_many::<String>("connector"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.get_many::<String>("os"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    Ok((
        OsChain::new(conn_iter, os_iter)?,
        matches.get_one::<String>("process").unwrap(),
        matches.get_one::<String>("module").unwrap(),
    ))
}
