#include "memflow.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(3);

	Inventory *inv = inventory_scan();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	ConnectorInstanceArcBox conn;
	if (!inventory_create_connector(inv, conn_name, conn_arg, &conn)) {
		for (int i = 0; i < 1000 * 1000 * 1000; i++) {
			uint8_t buffer[0x1000];
			PhysicalReadData read_data = {
				{ 0x1000, 1, 0 },
				{ buffer, 0x1000 }
			};
			conn.vtbl_physicalmemory->phys_read_raw_list(conn.instance.inner.instance, &read_data, 1);
			printf("Read: %lx\n", *(uint64_t *)buffer);
		}
		conn.instance.inner.drop(conn.instance.inner.instance);
		printf("conn dropped!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
