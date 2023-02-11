/**

This example demonstrates how to read the contents of a module from a process.

To read from a specific module the following steps have to be done:

  - Create an inventory and let it search for plugins in the system
  - Load the plugins to access physical memory and the operating system
  	(by default the `qemu` plugin and `win32` plugin are being used)
  - Find the process by the specified name
  - Find the module_info for the given module in the process
  - Allocate a buffer which will fit the entire module
  - Read the entire module into the buffer and ignore partial read errors
  - Write the contents of the retrieved buffer to the specified output location


Usage:

  ./module_dump.out kvm :: win32 :: notepad.exe notepad.exe notepad.exe.bin

*/
#include "memflow.h"

#include <stdio.h>
#include <string.h>
#include <errno.h>

int main(int argc, char *argv[]) {

	int ret = 0;

	// enable info level logging
	log_init(4);

	// load all available plugins
	Inventory *inventory = inventory_scan();
	printf("inventory initialized: %p\n", inventory);

	const char *conn_name = argc > 1 ? argv[1] : "qemu";
	const char *conn_arg = argc > 2 ? argv[2] : "";
	const char *os_name = argc > 3 ? argv[3]: "win32";
	const char *os_arg = argc > 4? argv[4]: "";
	const char *target_proc = argc > 5? argv[5]: "notepad.exe";
	const char *target_module = argc > 6? argv[6]: "notepad.exe";
	const char *output_file = argc > 7? argv[7]: "notepad.exe.bin";

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

		// find the module by its name
		ModuleInfo module_info;
		if (!(ret = mf_processinstance_module_by_name(&target_process, STR(target_module), &module_info))) {
			printf("%s module found: 0x%lx] 0x%lx %s %s\n", target_proc, module_info.address,
						module_info.base, module_info.name, module_info.path);

			// read module into buffer, in this case -2 / -3 are partial read/write errors
			void *module_buffer = malloc(module_info.size);
			ret = mf_processinstance_read_raw_into(&target_process, module_info.base, MUT_SLICE(u8, module_buffer, module_info.size));
			if (ret == -2) {
				printf("%s warning: %s] module only read partially\n", target_proc, target_module);
			}

			// module has been read
			printf("%s read module: %s] read 0x%lx bytes\n", target_proc, target_module, module_info.size);

			// write the buffer to the specified location
			FILE *file = fopen(output_file, "wb");
			if (file) {
				fwrite(module_buffer, module_info.size, 1, file);
				fclose(file);
				printf("dumped 0x%lx bytes to %s\n", module_info.size, output_file);
			} else {
				printf("unable to open output file %s: %s\n", output_file, strerror(errno));
			}

			free(module_buffer);
		} else {
			printf("unable to find module: %s\n", target_module);
			log_debug_errorcode(ret);
		}

		// cleanup the processinstance
		mf_processinstance_drop(target_process);
	} else {
		printf("unable to find process: %s\n", target_proc);
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
