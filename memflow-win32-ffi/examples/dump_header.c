#include "memflow_win32.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	log_init(1);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	const char *proc_name = argc > 3? argv[3]: "lsass.exe";
	const char *dll_name = argc > 4? argv[4]: "ntdll.dll";

	CloneablePhysicalMemoryObj *conn = inventory_create_connector(inv, conn_name, conn_arg);
	printf("conn: %p\n", conn);

	if (conn) {
		Kernel *kernel = kernel_build(conn);
		printf("Kernel: %p\n", kernel);
		Win32Version ver = kernel_winver(kernel);
		printf("major: %d\n", ver.nt_major_version);
		printf("minor: %d\n", ver.nt_minor_version);
		printf("build: %d\n", ver.nt_build_number);

		Win32Process *process = kernel_into_process(kernel, proc_name);

		if (process) {
			Win32ModuleInfo *module = process_module_info(process, dll_name);

			if (module) {
				OsProcessModuleInfoObj *obj = module_info_trait(module);
				Address base = os_process_module_base(obj);
				os_process_module_free(obj);
				VirtualMemoryObj *virt_mem = process_virt_mem(process);

				char header[256];
				if (!virt_read_raw_into(virt_mem, base, header, 256)) {
					printf("Read successful!\n");
					for (int o = 0; o < 8; o++) {
						for (int i = 0; i < 32; i++) {
							printf("%2hhx ", header[o * 32 + i]);
						}
						printf("\n");
					}
				} else {
					printf("Failed to read!\n");
				}

				virt_free(virt_mem);
			}

			process_free(process);
		}
	}

	inventory_free(inv);

	return 0;
}
