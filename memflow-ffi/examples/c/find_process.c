#include "memflow.h"

#include <stdio.h>
#include <string.h>

struct FindProcessContext {
	OsInstance *os;
	const char *name;
	ProcessInstance *target_process;
	bool found;
};

bool find_process(struct FindProcessContext *find_context, Address addr) {

	if (find_context->found) {
		return false;
	}

	if (mf_osinstance_process_by_address(
				find_context->os,
				addr,
				find_context->target_process
			)) {
		return true;
	}

	const struct ProcessInfo *info = mf_processinstance_info(find_context->target_process);

	if (!strcmp(info->name, find_context->name)) {
		// abort iteration
		find_context->found = true;
		return false;
	}

	mf_processinstance_drop(*find_context->target_process);

	// continue iteration
	return true;
}

int main(int argc, char *argv[]) {

	int ret = 0;

	// enable info level logging
	log_init(3);

	// load all available plugins
	Inventory *inventory = inventory_scan();
	printf("inventory initialized: %p\n", inventory);

	const char *conn_name = argc > 1 ? argv[1] : "qemu";
	const char *conn_arg = argc > 2 ? argv[2] : "";
	const char *os_name = argc > 3 ? argv[3]: "win32";
	const char *os_arg = argc > 4? argv[4]: "";
	const char *target_proc = argc > 5? argv[5]: "notepad.exe";

	ConnectorInstance connector, *conn = conn_name[0] ? &connector : NULL;

	// initialize the connector plugin
	if (conn) {
		if (inventory_create_connector(inventory, conn_name, conn_arg, conn)) {
			log_error("unable to initialize connector");
			inventory_free(inventory);
			return 1;
		}

		printf("connector initialized: %p\n", connector.container.instance.instance);
	}

	// initialize the OS plugin
	OsInstance os;
	if (inventory_create_os(inventory, os_name, os_arg, conn, &os)) {
		log_error("unable to initialize os plugin");
		inventory_free(inventory);
		return 1;
	}

	printf("os plugin initialized: %p\n", os.container.instance.instance);

	// find a specific process based on it's name.
	// this can easily be replaced by process_by_name but
	// is being used here as a demonstration.
	ProcessInstance target_process;
	struct FindProcessContext find_context = {
			&os,
			target_proc,
			&target_process,
			false,
	};

	mf_osinstance_process_address_list_callback(&os, CALLBACK(Address, &find_context, find_process));

	if (find_context.found) {
		const struct ProcessInfo *info = mf_processinstance_info(&target_process);

		printf("%s process found: 0x%lx] %d %s %s\n", target_proc, info->address,
					 info->pid, info->name, info->path);

		mf_processinstance_drop(target_process);
	} else {
		printf("Unable to find %s\n", target_proc);
	}

	// find a specific process based on its name
	// via process_by_name
	if (!(ret = mf_osinstance_process_by_name(&os, STR(target_proc), &target_process))) {
		const struct ProcessInfo *info = mf_processinstance_info(&target_process);

		printf("%s process found: 0x%lx] %d %s %s\n", target_proc, info->address,
					 info->pid, info->name, info->path);

		mf_processinstance_drop(target_process);
	} else {
		printf("Unable to find %s\n", target_proc);
		log_debug_errorcode(ret);
	}

	// This will also free the connector here
	// as it was _moved_ into the os by `inventory_create_os`
	mf_osinstance_drop(os);
	log_info("os plugin/connector freed");

	inventory_free(inventory);
	log_info("inventory freed");

	return 0;
}
