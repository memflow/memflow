use log::Level;

use memflow_core::connector::ConnectorArgs;
use memflow_pcileech::create_connector;

fn main() {
    simple_logger::init_with_level(Level::Trace).unwrap();
    create_connector(&ConnectorArgs::new()).unwrap();
}
