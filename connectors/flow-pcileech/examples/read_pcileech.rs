use log::Level;

use flow_pcileech::*;

fn main() {
    simple_logger::init_with_level(Level::Trace).unwrap();
    Memory::new().unwrap();
}
