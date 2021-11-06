#include "memflow.h"

#include <stdio.h>

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(3);

	Inventory *inv = inventory_scan();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1 ? argv[1] : "kvm";
	const char *conn_arg = argc > 2 ? argv[2] : "";

	ConnectorInstanceArcBox conn;
	if (!inventory_create_connector(inv, conn_name, conn_arg, &conn)) {
		for (int i = 0; i < 1000 * 1000; i++) {
			uint8_t buffer[0x1000];

			ConnectorInstanceArcBox cloned = connectorinstance_clone(&conn);

			connectorinstance_arc_box_drop(cloned);
			// mf_connector_phys_read_raw_list(conn, &read_data, 1);
			/*ConnectorInstance conn_cloned;
			mf_clone_connector(&conn, &conn_cloned);

			// most simple read
			PhysicalReadData read_data_0 = {{0x1000, 1, 0}, {buffer, 0x1000}};
			conn_cloned.vtbl_physicalmemory->phys_read_raw_list(
					this(&conn), (struct CSliceMut_PhysicalReadData){&read_data_0, 1});
			// printf("Read: %lx\n", *(uint64_t *)buffer);

			// regular read_into
			conn_cloned.vtbl_physicalmemory->phys_read_raw_into(
					this(&conn), (struct PhysicalAddress){0x1000, 1, 0},
					(struct CSliceMut_u8){buffer, sizeof(buffer)});
			// printf("Read: %lx\n", *(uint64_t *)buffer);

			mf_connector_free(conn_cloned);*/
		}

		connectorinstance_arc_box_drop(conn);
		printf("conn dropped!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
