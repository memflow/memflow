//use gdb_remote::GdbRemote;
//use std::io::{stdout, Write};
//use std::net::{SocketAddr, ToSocketAddrs};

use std::time::Instant;

use qemu_connector::QemuMonitor;

fn main() {
    // TODO: fork this lib and add missing features, maybe?
    /*
    let mut gdb = GdbRemote::new();
    gdb.connect(("127.0.0.1", 1234)).unwrap();

    let mut res = Vec::<u8>::with_capacity(8);
    let size = gdb.get_memory(&mut res, 0xfffff78900014500, 8).unwrap();

    println!("{:?}", size);
    println!("{:?}", res);
    */

    let mut mon = match QemuMonitor::connect("/tmp/qemu-monitor-socket") {
        Ok(s) => s,
        Err(e) => {
            println!("Couldn't connect: {:?}", e);
            return;
        }
    };

    let r = mon.exec_cmd("xp /64b 0xF000").unwrap();
    println!("{}", r);

    let start = Instant::now();
    let mut counter = 0;
    loop {
        mon.exec_cmd("xp /4b 0x1000").unwrap();

        counter += 1;
        if (counter % 1000) == 0 {
            let elapsed = start.elapsed().as_millis() as f64;
            if elapsed > 0.0 {
                println!("{} reads/sec", (counter as f64) / elapsed * 1000.0);
                println!("{} ms/read", elapsed / (counter as f64));
            }
        }
    }
}
