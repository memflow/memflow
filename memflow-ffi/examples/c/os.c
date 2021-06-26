#include <inttypes.h>
#include <stdio.h>

#include "memflow.h"
#include "memflow_support.h"

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

  // physical read on os
  PhysicalMemoryInstance *phys_mem = os_phys_mem(&os);
  if (phys_mem != 0) {
    PhysicalMemoryMetadata metadata = phys_metadata(phys_mem);
    printf("PhysicalMemoryMetadata{ size: %" PRIu64 ", writable: %s }\n",
           (uint64_t)metadata.size, metadata.readonly ? "true" : "false");
  }

  // virtual read on os
  /*
  VirtualMemoryInstance *virt_mem = os_virt_mem(&os);
  printf("virt_mem: 0x%x\n", virt_mem);
  if (virt_mem != 0) {
      printf("instance: 0x%x\n", phys_mem->instance);
      printf("vt: 0x%x\n", phys_mem->vtable);
      PhysicalMemoryMetadata metadata = phys_metadata(phys_mem);
      printf("PhysicalMemoryMetadata{ size: %U64d, writable: %s }\n",
  metadata.size, metadata.readonly ? "true" : "false");
  }
  */

  // os_drop will also free the connector here
  // as it was _moved_ into the os by `inventory_create_os`
  os_drop(&os);
  printf("os plugin/connector freed\n");

  inventory_free(inventory);
  printf("inventory freed\n");

  return 0;
}
