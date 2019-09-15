use clap::{App, Arg};
use std::time::Instant;

fn main() {
    let argv = App::new("flow-core")
        .version("0.1")
        .arg(
            Arg::with_name("socket")
                .short("s")
                .long("socket")
                .value_name("FILE")
                .help("bridge unix socket file")
                .takes_value(true),
        )
        .get_matches();

    // TODO: implement me
}
