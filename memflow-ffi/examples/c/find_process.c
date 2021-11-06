#include "memflow.h"

#include <stdio.h>
#include <string.h>

struct FindProcessContext {
	OsInstanceArcBox *os;
	const char *name;
	ProcessInstanceArcBox *target_process;
	bool found;
};

bool find_process(struct FindProcessContext *find_context, Address addr) {

	if (find_context->found) {
		return false;
	}

	if (osinstance_process_by_address(
				find_context->os,
				addr,
				find_context->target_process
			)) {
		return true;
	}

	const struct ProcessInfo *info = processinstance_info(find_context->target_process);

	if (!strcmp(info->name, find_context->name)) {
		// abort iteration
		find_context->found = true;
		return false;
	}

	processinstance_arc_box_drop(*find_context->target_process);

	// continue iteration
	return true;
}

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(2);

	// load all available plugins
	Inventory *inventory = inventory_scan();
	printf("inventory initialized: %p\n", inventory);

	const char *conn_name = argc > 1 ? argv[1] : "qemu_procfs";
	const char *conn_arg = argc > 2 ? argv[2] : "";
	const char *os_name = argc > 3 ? argv[3]: "win32";
	const char *os_arg = argc > 4? argv[4]: "";
	const char *target_proc = argc > 5? argv[5]: "notepad.exe";

	ConnectorInstanceArcBox connector, *conn = conn_name[0] ? &connector : NULL;

	// initialize the connector plugin
	if (conn) {
		if (inventory_create_connector(inventory, conn_name, conn_arg, conn)) {
			printf("unable to initialize connector\n");
			inventory_free(inventory);
			return 1;
		}

		printf("connector initialized: %p\n", connector.container.instance.instance);
	}

	// initialize the OS plugin
	OsInstanceArcBox os;
	if (inventory_create_os(inventory, os_name, os_arg, conn, &os)) {
		printf("unable to initialize os plugin\n");
		inventory_free(inventory);
		return 1;
	}

	printf("os plugin initialized: %p\n", os.container.instance.instance);

	// find a specific process based on it's name.
	// this can easily be replaced by process_by_name but
	// is being used here as a demonstration.
	ProcessInstanceArcBox target_process;
	struct FindProcessContext find_context = {
			&os,
			target_proc,
			&target_process,
			false,
	};

	osinstance_process_address_list_callback(&os, CALLBACK(Address, &find_context, find_process));

	if (find_context.found) {
		const struct ProcessInfo *info = processinstance_info(&target_process);

		printf("%s process found: 0x%lx] %d %s %s\n", target_proc, info->address,
					 info->pid, info->name, info->path);

		processinstance_arc_box_drop(target_process);
	} else {
		printf("Unable to find %s\n", target_proc);
	}

	// find a specific process based on its name
	// via process_by_name
	if (!osinstance_process_by_name(&os, STR(target_proc), &target_process)) {
		const struct ProcessInfo *info = processinstance_info(&target_process);

		printf("%s process found: 0x%lx] %d %s %s\n", target_proc, info->address,
					 info->pid, info->name, info->path);

		processinstance_arc_box_drop(target_process);
	} else {
		printf("Unable to find %s\n", target_proc);
	}

	// This will also free the connector here
	// as it was _moved_ into the os by `inventory_create_os`
	osinstance_arc_box_drop(os);
	printf("os plugin/connector freed\n");

	inventory_free(inventory);
	printf("inventory freed\n");

	return 0;
}
