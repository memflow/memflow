#include "memflow.h"

#include <stdio.h>
#include <string.h>

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

	// find a specific process based on its name via process_by_name
	ProcessInstance target_process;
	if (!(ret = mf_osinstance_process_by_name(&os, STR(target_proc), &target_process))) {
		const struct ProcessInfo *info = mf_processinstance_info(&target_process);

		printf("%s process found: 0x%lx] %d %s %s\n", target_proc, info->address,
					 info->pid, info->name, info->path);

		// iterate over all module info structs and collect them in a buffer
		COLLECT_CB(ModuleInfo, module_info);
		mf_processinstance_module_list_callback(&target_process, NULL, module_info);
		for (size_t i = 0; i < module_info_base.size; i++) {
			ModuleInfo *module_info = &((ModuleInfo *)module_info_base.buf)[i];
			printf("%s module found: 0x%lx] 0x%lx %s %s\n", target_proc, module_info->address,
						module_info->base, module_info->name, module_info->path);
		}
		free(module_info_base.buf);

		// cleanup the processinstance
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
