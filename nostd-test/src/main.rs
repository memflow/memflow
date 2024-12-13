#![no_std]
#![no_main]

// no-std-compat
extern crate no_std_compat as std;

#[macro_use]
extern crate smallvec;

use std::{convert::TryInto, ptr::addr_of, vec::Vec};

use memflow::prelude::v1::*;
use talc::*;

// setup global allocator
static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array(addr_of!(ARENA) as *mut [u8; 10000])) })
        .lock();

pub struct MemoryBackend {
    mem: Vec<u8>,
}

impl PhysicalMemory for MemoryBackend {
    fn phys_read_raw_iter(
        &mut self,
        MemOps { inp, mut out, .. }: PhysicalReadMemOps,
    ) -> Result<()> {
        inp.for_each(|CTup3(addr, meta_addr, mut data)| {
            let addr: usize = addr.to_umem().try_into().unwrap();
            let len = data.len();
            data.copy_from_slice(&self.mem[addr..(addr + len)]);
            opt_call(out.as_deref_mut(), CTup2(meta_addr, data));
        });
        Ok(())
    }

    fn phys_write_raw_iter(
        &mut self,
        MemOps { inp, mut out, .. }: PhysicalWriteMemOps,
    ) -> Result<()> {
        inp.for_each(|CTup3(addr, meta_addr, data)| {
            let addr: usize = addr.to_umem().try_into().unwrap();
            let len = data.len();
            self.mem[addr..(addr + len)].copy_from_slice(&data);
            opt_call(out.as_deref_mut(), CTup2(meta_addr, data));
        });
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            max_address: (self.mem.len() - 1).into(),
            real_size: self.mem.len() as umem,
            readonly: false,
            ideal_batch_size: u32::MAX,
        }
    }
}

#[allow(unused)]
fn special_read(mem: &mut impl MemoryView, addr: Address) -> Result<u8> {
    mem.read(addr).data()
}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut mem = MemoryBackend {
        mem: vec![0xA; 128],
    };

    assert_eq!(
        special_read(&mut mem.phys_view(), 0x42.into()).unwrap(),
        0xA
    );

    0
}
