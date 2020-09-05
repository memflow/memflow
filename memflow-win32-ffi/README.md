# memflow-win32-ffi
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

The [memflow](https://github.com/memflow/memflow) win32 FFI crate provides an interface to the memflow-win32 API for C/C++. Currently a single `memflow_win32.h` file is generated aside from the dynamic library that can be used to interact with memflow.

This FFI library is intended to be used in combination with the [memflow-ffi](https://github.com/memflow/memflow/memflow-ffi) library.

A simple example that initializes the memflow-ffi and memflow-win32-ffi:
```cpp
#include "memflow_win32.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	log_init(1);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	CloneablePhysicalMemoryObj *conn =
        inventory_create_connector(inv, conn_name, conn_arg);
	printf("conn: %p\n", conn);

	if (conn) {
		Kernel *kernel = kernel_build(conn);
		printf("Kernel: %p\n", kernel);
		Win32Version ver = kernel_winver(kernel);
		printf("major: %d\n", ver.nt_major_version);
		printf("minor: %d\n", ver.nt_minor_version);
		printf("build: %d\n", ver.nt_build_number);

		kernel_free(kernel);
	}

	inventory_free(inv);

	return 0;
}
```

Additional examples can be found in the `examples` folder as well as in the [memflow-ffi](https://github.com/memflow/memflow/memflow-ffi) crate.
