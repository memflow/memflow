use std::time::Instant;

use flow_core::*;

fn main() {
    let mut conn = match flow_qemu_procfs::Memory::new() {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't open memory read context: {:?}", e);
            return;
        }
    };

    let mut mem = vec![0; 8];
    conn.phys_read(Address::from(0x1000), &mut mem).unwrap();
    println!("Received memory: {:?}", mem);

    let start = Instant::now();
    let mut counter = 0;
    loop {
        let mut buf = vec![0; 0x1000];
        conn.phys_read(Address::from(0x1000), &mut buf).unwrap();

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
