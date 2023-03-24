#include "memflow.h"

#include <stdio.h>

bool list_processes(OsInstance *os, Address addr) {

	int ret;

	ProcessInstance process;
	if ((ret = mf_osinstance_process_by_address(os, addr, &process))) {
		log_debug_errorcode(ret);
		return true;
	}

	const struct ProcessInfo *info = mf_processinstance_info(&process);

	ModuleInfo primary_module;
	if ((ret = mf_processinstance_primary_module(&process, &primary_module))) {
		// no primary module found, continue iteration - this should _never_ happen
		printf("%d\t%s\t0x%lx\tN/A\n", info->pid, info->name, info->address);
		log_debug_errorcode(ret);
		return true;
	}

	printf("%d\t%s\t0x%lx\t0x%lx\n", info->pid, info->name, info->address,
				 primary_module.address);

	// iterate over all module addresses and collect them in an array
	struct ModuleAddressInfo module_addresses[256];
	COLLECT_CB_INTO_ARR(ModuleAddressInfo, module_address, module_addresses);
	mf_processinstance_module_address_list_callback(&process, NULL, module_address);

	printf("Read %zu modules\n", module_address_base.size);

	// iterate over all module info structs and collect them in a buffer
	COLLECT_CB(ModuleInfo, module_info);
	mf_processinstance_module_list_callback(&process, NULL, module_info);
	printf("Read %zu modules\n", module_info_base.size);
	free(module_info_base.buf);

	// iterate over all imports and collect them in a buffer
	COLLECT_CB(ImportInfo, import_info);
	mf_processinstance_module_import_list_callback(&process, &primary_module, import_info);
	printf("Read %zu imports\n", import_info_base.size);
	free(import_info_base.buf);

	// iterate over all exports and collect them in a buffer
	COLLECT_CB(ExportInfo, exports);
	mf_processinstance_module_export_list_callback(&process, &primary_module, exports);
	printf("Read %zu exports\n", exports_base.size);
	free(exports_base.buf);

	// iterate over all sections and collect them in a buffer
	COLLECT_CB(SectionInfo, sections);
	mf_processinstance_module_section_list_callback(&process, &primary_module, sections);
	printf("Read %zu sections\n", sections_base.size);
	free(sections_base.buf);

	mf_processinstance_drop(process);

	return true;
}

int main(int argc, char *argv[]) {
	// enable debug level logging
	log_init(2);

	// load all available plugins
	Inventory *inventory = inventory_scan();

	printf("inventory initialized: %p\n", inventory);

	const char *conn_name = argc > 1 ? argv[1] : "qemu";
	const char *conn_arg = argc > 2 ? argv[2] : "";
	const char *os_name = argc > 3 ? argv[3]: "win32";
	const char *os_arg = argc > 4? argv[4]: "";

	ConnectorInstance connector, *conn = conn_name[0] ? &connector : NULL;

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
	OsInstance os;
	if (inventory_create_os(inventory, os_name, os_arg, conn, &os)) {
		printf("unable to initialize os plugin\n");
		inventory_free(inventory);
		return 1;
	}

	printf("os plugin initialized: %p\n", os.container.instance.instance);

	// iterate over all processes and print them manually
	printf("Pid\tNAME\tADDRESS\tMAIN_MODULE\n");
	mf_osinstance_process_address_list_callback(&os, CALLBACK(Address, &os, list_processes));

	// count all processes
	COUNT_CB(Address, process_address);
	mf_osinstance_process_address_list_callback(&os, process_address);
	printf("Counted %zu processes\n", process_address_count);

	// iterate over all process info structs and collect them in an array
	struct ProcessInfo process_info[256];
	COLLECT_CB_INTO_ARR(ProcessInfo, process_info_cb, process_info);
	mf_osinstance_process_info_list_callback(&os, process_info_cb);
	printf("Read %zu process infos\n", process_info_cb_base.size);

	// This will also free the connector here
	// as it was _moved_ into the os by `inventory_create_os`
	mf_osinstance_drop(os);
	printf("os plugin/connector freed\n");

	inventory_free(inventory);
	printf("inventory freed\n");

	return 0;
}
