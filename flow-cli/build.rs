#[macro_use]
extern crate clap;
use clap::{App, Shell};

fn main() {
    let outdir = match std::env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let yaml = load_yaml!("src/cli.yml");
    let mut app = App::from(yaml);
    app.gen_completions("flow-cli", Shell::Bash, outdir.clone());
    app.gen_completions("flow-cli", Shell::Fish, outdir.clone());
    app.gen_completions("flow-cli", Shell::Zsh, outdir.clone());
}
