#include "../memflow_core.h"
#include <stdio.h>

int main() {
	log_init(4);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	ConnectorInstance *conn = inventory_create_connector(inv, "kvm", "172782");
	printf("conn: %p\n", conn);

	if (conn) {
		PhysicalMemoryObj *phys_mem = downcast_cloneable(conn);
		printf("phys_mem: %p\n", phys_mem);

		PhysicalAddress addr = {
			.address = 0x30000
		};

		phys_write_u64(phys_mem, addr, 0);

		phys_free(phys_mem);

		connector_free(conn);
		printf("conn freed!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
