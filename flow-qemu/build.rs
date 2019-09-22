extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .src_prefix("bridge/schema")
        .file("bridge/schema/bridge.capnp")
        .run()
        .expect("compiling schema");
}
