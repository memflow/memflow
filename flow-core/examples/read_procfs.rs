use std::time::Instant;

use flow_core::*;
use flow_core::address::{Address, Length};
use flow_core::mem::PhysicalRead;
use flow_core::connector::qemu_procfs;

fn main() {
    let mut conn = match qemu_procfs::Memory::new() {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't open memory read context: {:?}", e);
            return;
        }
    };

    let mem = conn.phys_read(Address::from(0x1000), len!(8)).unwrap();
    println!("Received memory: {:?}", mem);

    //bridge.read_registers().unwrap();

    let start = Instant::now();
    let mut counter = 0;
    loop {
        //let r = bridge.read_memory(0x1000, 0x1000).unwrap();
        //bridge.write_memory(0x1000, &r).unwrap();
        conn
            .phys_read(Address::from(0x1000), len!(0x1000))
            .unwrap();

        counter += 1;
        if (counter % 10000) == 0 {
            let elapsed = start.elapsed().as_millis() as f64;
            if elapsed > 0.0 {
                println!("{} reads/sec", (f64::from(counter)) / elapsed * 1000.0);
                println!(
                    "{} reads/frame",
                    (f64::from(counter)) / elapsed * 1000.0 / 60.0
                );
                println!("{} ms/read", elapsed / (f64::from(counter)));
            }
        }
    }
}
