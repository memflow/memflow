use std::time::Instant;

use log::{info, Level};

use memflow_core::*;

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut conn = match memflow_qemu_procfs::Memory::new() {
        Ok(br) => br,
        Err(e) => {
            info!("couldn't open memory read context: {:?}", e);
            return;
        }
    };

    let mut mem = vec![0; 8];
    conn.phys_read_raw_into(Address::from(0x1000).into(), &mut mem)
        .unwrap();
    info!("Received memory: {:?}", mem);

    let start = Instant::now();
    let mut counter = 0;
    loop {
        let mut buf = vec![0; 0x1000];
        conn.phys_read_raw_into(Address::from(0x1000).into(), &mut buf)
            .unwrap();

        counter += 1;
        if (counter % 10000) == 0 {
            let elapsed = start.elapsed().as_millis() as f64;
            if elapsed > 0.0 {
                info!("{} reads/sec", (f64::from(counter)) / elapsed * 1000.0);
                info!(
                    "{} reads/frame",
                    (f64::from(counter)) / elapsed * 1000.0 / 60.0
                );
                info!("{} ms/read", elapsed / (f64::from(counter)));
            }
        }
    }
}
