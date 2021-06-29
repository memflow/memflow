#include "memflow.h"
#include "memflow_support.h"

#include <stdio.h>
#include <string.h>

struct FindProcessContext {
  OsInstance *os;
  const char *name;
  ProcessInstance *target_process;
  bool found;
};

bool find_process(struct FindProcessContext *find_context, Address addr) {
  if (find_context->os->vtbl_osinner->process_by_address(
          this(find_context->os), addr, ctx(find_context->os),
          find_context->target_process) < 0) {
    return true;
  }

  const struct ProcessInfo *info =
      find_context->target_process->vtbl_process->info(
          this(find_context->target_process));
  if (!strcmp(info->name, find_context->name)) {
    // abort iteration
    find_context->found = true;
    return false;
  }

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

  // find a specific process based on it's name.
  // this can easily be replaced by process_by_name but
  // is being used here as a demonstration.
  ProcessInstance target_process;
  struct FindProcessContext find_context = {
      &os,
      "Calculator.exe",
      &target_process,
      false,
  };
  os.vtbl_osinner->process_address_list_callback(
      this(&os), mf_cb_address(&find_context, find_process));
  if (find_context.found) {
    const struct ProcessInfo *info =
        target_process.vtbl_process->info(this(&target_process));

    printf("Calculator.exe process found: 0x%x] %d %s %s\n", info->address,
           info->pid, info->name, info->path);
  } else {
    printf("Unable to find Calculator.exe\n");
  }

  // find a specific process based on its name
  // via process_by_name
  if (os.vtbl_osinner->process_by_name(this(&os), str("Calculator.exe"),
                                       ctx(&os), &target_process) == 0) {
    const struct ProcessInfo *info =
        target_process.vtbl_process->info(this(&target_process));

    printf("Calculator.exe process found: 0x%x] %d %s %s\n", info->address,
           info->pid, info->name, info->path);
  } else {
    printf("Unable to find Calculator.exe\n");
  }

  // mf_os_free will also free the connector here
  // as it was _moved_ into the os by `inventory_create_os`
  mf_os_free(os);
  printf("os plugin/connector freed\n");

  inventory_free(inventory);
  printf("inventory freed\n");

  return 0;
}
