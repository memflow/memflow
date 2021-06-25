#include "memflow.h"
#include <stdio.h>

#define mf_connector_free(conn) (conn).instance.inner.drop((conn).instance.inner.instance)

#define mf_connector_phys_read_raw_list(conn, ...) (conn).vtbl_physicalmemory->phys_read_raw_list((conn).instance.inner.instance, __VA_ARGS__);
#define mf_connector_phys_write_raw_list(conn, ...) (conn).vtbl_physicalmemory->phys_write_raw_list((conn).instance.inner.instance, __VA_ARGS__);
#define mf_connector_metadata(conn) (conn).vtbl_physicalmemory->metadata((conn).instance.inner.instance);
#define mf_connector_set_mem_map(conn, ...) (conn).vtbl_physicalmemory->set_mem_map((conn).instance.inner.instance, __VA_ARGS__);

#define mf_clone(conn) ((conn).vtbl_clone)
#define mf_phys(conn) ((conn).vtbl_physicalmemory)
#define this(obj) ((obj)->instance.inner.instance)
#define ctx(obj) (&(obj)->instance.ctx)

void mf_clone_connector(const ConnectorInstanceArcBox *conn, ConnectorInstanceArcBox *out) {
	(*out).vtbl_clone = conn->vtbl_clone;
	(*out).vtbl_connectorcpustateinner = conn->vtbl_connectorcpustateinner;
	(*out).vtbl_physicalmemory = conn->vtbl_physicalmemory;
	(*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}

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
			
			//mf_connector_phys_read_raw_list(conn, &read_data, 1);
			ConnectorInstanceArcBox conn_cloned;
			mf_clone_connector(&conn, &conn_cloned);

			// most simple read
			PhysicalReadData read_data_0 = {
				{ 0x1000, 1, 0 },
				{ buffer, 0x1000 }
			};
			conn_cloned.vtbl_physicalmemory->phys_read_raw_list(
				this(&conn),
				(struct CSliceMut_PhysicalReadData){ &read_data_0, 1 });
			printf("Read: %lx\n", *(uint64_t *)buffer);

			// regular read_into
			conn_cloned.vtbl_physicalmemory->phys_read_raw_into(
				this(&conn),
				(struct PhysicalAddress){ 0x1000, 1, 0 },
				(struct CSliceMut_u8){ buffer, sizeof(buffer) });
			printf("Read: %lx\n", *(uint64_t *)buffer);

			mf_connector_free(conn_cloned);
		}
		//conn.instance.inner.drop(conn.instance.inner.instance);
		mf_connector_free(conn);
		printf("conn dropped!\n");
	}

	inventory_free(inv);
	printf("inv freed!\n");

	return 0;
}
