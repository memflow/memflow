#include "memflow.h"
#include "memflow_support.h"

#include <stdio.h>

bool list_processes(OsInstance *os, Address addr) {
  ProcessInstance process;
  if (os->vtbl_osinner->process_by_address(this(os), addr, ctx(os), &process) <
      0) {
    return true;
  }

  const struct ProcessInfo *info = process.vtbl_process->info(this(&process));

  ModuleInfo primary_module;
  if (process.vtbl_process->primary_module(this(&process), &primary_module) <
      0) {
    // no primary module found, continue iteration - this should _never_ happen
    printf("%d\t%s\t0x%x\tN/A\n", info->pid, info->name, info->address);
    return true;
  }

  printf("%d\t%s\t0x%x\t0x%x\n", info->pid, info->name, info->address,
         primary_module.address);

  // iterate over all module addresses and collect them in an array
  struct ModuleAddressInfo module_address[256];
  struct CollectModuleAddressInfoContext modulesctx = {module_address, 256, 0,
                                                       0};
  process.vtbl_process->module_address_list_callback(
      this(&process), NULL,
      mf_cb_module_address_info(&modulesctx, collect_module_address_info));
  printf("Read %d of %d modules\n", modulesctx.read, modulesctx.total);

  // iterate over all module info structs and collect them in an array
  struct ModuleInfo module_info[64];
  struct CollectModuleInfoContext modulesinfoctx = {module_info, 64, 0, 0};
  process.vtbl_process->module_list_callback(
      this(&process), NULL,
      mf_cb_module_info(&modulesinfoctx, collect_module_info));
  printf("Read %d of %d modules\n", modulesinfoctx.read, modulesinfoctx.total);

  // iterate over all imports and collect them in an array
  struct ImportInfo import_info[64];
  struct CollectImportInfoContext importsctx = {import_info, 64, 0, 0};
  process.vtbl_process->module_import_list_callback(
      this(&process), &primary_module,
      mf_cb_import_info(&importsctx, collect_import_info));
  printf("Read %d of %d imports\n", importsctx.read, importsctx.total);

  // iterate over all exports and collect them in an array
  struct ExportInfo export_info[64];
  struct CollectExportInfoContext exportsctx = {export_info, 64, 0, 0};
  process.vtbl_process->module_export_list_callback(
      this(&process), &primary_module,
      mf_cb_export_info(&exportsctx, collect_export_info));
  printf("Read %d of %d exports\n", exportsctx.read, exportsctx.total);

  // iterate over all sections and collect them in an array
  struct SectionInfo section_info[64];
  struct CollectSectionInfoContext sectionsctx = {section_info, 64, 0, 0};
  process.vtbl_process->module_section_list_callback(
      this(&process), &primary_module,
      mf_cb_section_info(&sectionsctx, collect_section_info));
  printf("Read %d of %d sections\n", sectionsctx.read, sectionsctx.total);

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

  // iterate over all processes and print them manually
  printf("Pid\tNAME\tADDRESS\tMAIN_MODULE\n");
  os.vtbl_osinner->process_address_list_callback(
      this(&os), mf_cb_address(&os, list_processes));

  // iterate over all processes and collect them in an array
  Address process_address[256];
  struct CollectAddressContext processesctx = {process_address, 256, 0, 0};
  os.vtbl_osinner->process_address_list_callback(
      this(&os), mf_cb_address(&processesctx, collect_address));
  printf("Read %d of %d processes\n", processesctx.read, processesctx.total);

  // iterate over all process info structs and collect them in an array
  struct ProcessInfo process_info[256];
  struct CollectProcessInfoContext processesinfoctx = {process_info, 256, 0, 0};
  os.vtbl_osinner->process_info_list_callback(
      this(&os), mf_cb_process_info(&processesinfoctx, collect_process_info));
  printf("Read %d of %d process infos\n", processesinfoctx.read,
         processesinfoctx.total);

  // mf_os_free will also free the connector here
  // as it was _moved_ into the os by `inventory_create_os`
  mf_os_free(os);
  printf("os plugin/connector freed\n");

  inventory_free(inventory);
  printf("inventory freed\n");

  return 0;
}
