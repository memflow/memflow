#include "../bindings/c/memflow_core.h"
#include <stdio.h>

int main() {
	log_init(4);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	CloneablePhysicalMemoryObj *conn = inventory_create_connector(inv, "kvm", "56353");
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
