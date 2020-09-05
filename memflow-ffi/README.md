# memflow-ffi
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

The [memflow](https://github.com/memflow/memflow) FFI crate provides an interface to the memflow API for C/C++. Currently a single `memflow.h` file is generated aside from the dynamic library that can be used to interact with memflow.

A simple example that initializes the library:
```cpp
#include "memflow.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	log_init(4);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	CloneablePhysicalMemoryObj *conn =
        inventory_create_connector(inv, conn_name, conn_arg);
	printf("conn: %p\n", conn);

	if (conn) {
		PhysicalMemoryObj *phys_mem = downcast_cloneable(conn);
		printf("phys_mem: %p\n", phys_mem);

		uint64_t read = phys_read_u64(phys_mem, addr_to_paddr(0x30000));

		printf("Read: %lx\n", read);

		phys_free(phys_mem);

		connector_free(conn);
		printf("conn freed!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
```

Additional examples can be found in the `examples` folder as well as in the [memflow-win32-ffi](https://github.com/memflow/memflow/memflow-win32-ffi) crate.
