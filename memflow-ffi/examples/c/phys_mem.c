#include "memflow.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(3);

	Inventory *inv = inventory_scan();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	ConnectorInstance conn;
	if (!inventory_create_connector(inv, conn_name, conn_arg, &conn)) {
		uint64_t read = phys_read_u64(&conn, addr_to_paddr(0x30000));

		printf("Read: %lx\n", read);

		connector_drop(&conn);
		printf("conn dropped!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
