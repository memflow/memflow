#include "memflow_win32.h"
#include <stdio.h>

int main(int argc, char *argv[]) {
	log_init(1);

	ConnectorInventory *inv = inventory_try_new();
	printf("inv: %p\n", inv);

	const char *conn_name = argc > 1? argv[1]: "kvm";
	const char *conn_arg = argc > 2? argv[2]: "";

	CloneablePhysicalMemoryObj *conn = inventory_create_connector(inv, conn_name, conn_arg);
	printf("conn: %p\n", conn);

	if (conn) {
		Kernel *kernel = kernel_build(conn);
		printf("Kernel: %p\n", kernel);
		Win32Version ver = kernel_winver(kernel);
		printf("major: %d\n", ver.nt_major_version);
		printf("minor: %d\n", ver.nt_minor_version);
		printf("build: %d\n", ver.nt_build_number);

		Win32ProcessInfo *processes[512];
		size_t process_count = kernel_process_info_list(kernel, processes, 512);

		printf("Process List:\n");
		printf("%-8s | %-16s | %-16s | %-12s | %-5s\n", "PID", "Name", "Base", "DTB", "Wow64");

		for (size_t i = 0; i < process_count; i++) {
			Win32ProcessInfo *process = processes[i];
			OsProcessInfoObj *info = process_info_trait(process);
			char name[32];
			os_process_info_name(info, name, 32);
			
			printf("%-8d | %-16s | %-16lx | %-12lx | %-5s\n",
					os_process_info_pid(info),
					name,
					process_info_section_base(process),
					process_info_dtb(process),
					process_info_wow64(process)? "Yes" : "No"
				);

			os_process_info_free(info);
			process_info_free(process);
		}

		kernel_free(kernel);
	}

	inventory_free(inv);

	return 0;
}
