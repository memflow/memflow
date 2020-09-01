#include "memflow.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	log_init(4);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	CloneablePhysicalMemoryObj *conn = inventory_create_connector(inv, conn_name, conn_arg);
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
