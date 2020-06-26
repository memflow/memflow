use log::Level;

use flow_pcileech::*;

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();
    Memory::new().unwrap();
}
