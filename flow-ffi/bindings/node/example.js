var ffi = require('ffi-napi');
//var assert = require('assert');

var memflow = ffi.Library("../../target/release/libmemflow", {
    "bridge_connect": [ "pointer", [ "char *" ]],
    "bridge_free": [ "void", [ "pointer" ] ],
    // bridge_phys_read
    // bridge_virt_read
    "win32_init_bridge": [ "pointer", [ "pointer" ] ],
});

console.log("trying to connect");
var ctx = memflow.bridge_connect(Buffer.from("tcp:127.0.0.1:8181,nodelay"));
if (ctx != null) {
    console.log("connected");
    var win = memflow.win32_init_bridge(ctx);
    if (win != null) {
        console.log("win32 initialized");
    } else {
        console.log("win32 failed to initialize");
    }
    memflow.bridge_free(ctx);
    console.log("disconnected");
} else {
    console.log("connection failed");
}
