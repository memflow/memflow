#include "memflow.h"
#include "memflow_support.h"
#include <stdio.h>

bool list_modules(ProcessInstance *process, struct ModuleInfo module_info) {
  printf("0x%x\t%s\n", module_info.address, module_info.name);
  return true;
}

bool list_processes(OsInstance *os, Address addr) {
  ProcessInstance process;
  if (os->vtbl_osinner->process_by_address(this(os), addr, ctx(os), &process) <
      0) {
    return true;
  }

  const struct ProcessInfo *info = process.vtbl_process->info(this(&process));

  /*
          ModuleInfo module;
          if (process.vtbl_process->primary_module(this(&process), &module) >=
     0) { printf("%d\t%s\t0x%x\t0x%x\n", info->pid, info->name, info->address,
     module.address); } else { printf("%d\t%s\t0x%x\t%s\n", info->pid,
     info->name, info->address, "N/A");
          }
  */

  process.vtbl_process->module_list_callback(
      this(&process), 0, mf_cb_module_info(&process, list_modules));

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

  // initialize the connector plugin
  ConnectorInstance connector;
  if (inventory_create_connector(inventory, conn_name, conn_arg, &connector) <
      0) {
    printf("unable to initialize connector\n");
    inventory_free(inventory);
    return 1;
  }
  printf("connector initialized: %p\n", this(&connector));

  // initialize the OS plugin
  OsInstance os;
  if (inventory_create_os(inventory, "win32", "", connector, &os) < 0) {
    printf("unable to initialize os plugin\n");
    mf_connector_free(connector);
    inventory_free(inventory);
    return 1;
  }
  printf("os plugin initialized: %p\n", this(&os));

  // iterate over all processes and print them
  printf("Pid\tNAME\tADDRESS\tMAIN_MODULE\n");
  os.vtbl_osinner->process_address_list_callback(
      this(&os), mf_cb_address(&os, list_processes));

  // mf_os_free will also free the connector here
  // as it was _moved_ into the os by `inventory_create_os`
  mf_os_free(os);
  printf("os plugin/connector freed\n");

  inventory_free(inventory);
  printf("inventory freed\n");

  return 0;
}
