#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

void connector_free(void *thisptr);

int32_t connector_inventory_add_dir(void *thisptr, const char *dir);

void *connector_inventory_create_connector(void *thisptr, const char *name, const char *args);

void connector_inventory_free(void *thisptr);

void *connector_inventory_try_new();

void *connector_inventory_with_path(const char *path);

void log_init(int32_t level);

} // extern "C"
