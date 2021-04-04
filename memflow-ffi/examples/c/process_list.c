#include "memflow.h"
#include <stdio.h>

bool list_processes(OsInstance *os, Address addr) {
	ProcessInfo proc_info;
	if (os->vtable.os->process_info_by_address(os->instance, addr, &proc_info) < 0) {
		return true;
	}

	printf("%d\t%s\t0x%x\n", proc_info.pid, proc_info.name, proc_info.address);

	return true;
}

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(2);

	// load all available plugins
	Inventory *inventory = inventory_scan();
	printf("inventory initialized: %p\n", inventory);

	const char *conn_name = argc > 1 ? argv[1]: "qemu_procfs";
	const char *conn_arg = argc > 2 ? argv[2]: "";

	// initialize the connector plugin
	ConnectorInstance connector;
	if (inventory_create_connector(inventory, conn_name, conn_arg, &connector) < 0) {
		printf("unable to initialize connector\n");
		inventory_free(inventory);
		return 1;
	}
	printf("connector initialized: %p\n", connector);

	// initialize the OS plugin
	OsInstance os;
	if (inventory_create_os(inventory, "win32", "", connector, &os) < 0) {
		printf("unable to initialize os plugin\n");
		connector_drop(&connector);
		inventory_free(inventory);
		return 1;
	}
	printf("os plugin initialized: %p\n", os);

	// iterate over all processes and print them
	printf("Pid\tNAME\tADDRESS\n");
	OpaqueCallback_Address cb;
	cb.context = &os;
	cb.func = (bool (*)(void *, Address))list_processes;
	os.vtable.os->process_address_list_callback(os.instance, cb);

	// os_drop will also free the connector here
	// as it was _moved_ into the os by `inventory_create_os`
	os_drop(&os);
	printf("os plugin/connector freed\n");

	inventory_free(inventory);
	printf("inventory freed\n");

	return 0;
}
