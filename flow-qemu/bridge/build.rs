extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/bridge.capnp")
        .run()
        .expect("compiling schema");
}
