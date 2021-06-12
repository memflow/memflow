#ifndef MEMFLOW_SUPPORT_H
#define MEMFLOW_SUPPORT_H

#include "memflow.h"

// `ConnectorInstance` helpers
/*
int32_t connector_read_raw_list(struct ConnectorInstance *phys, struct PhysicalReadData *read_data, uintptr_t read_data_count) {
    return phys->vtable.phys->phys_read_raw_list(phys->instance, read_data, read_data_count);
}

int32_t connector_write_raw_list(struct ConnectorInstance *phys, const struct PhysicalWriteData *write_data, uintptr_t write_data_count) {
    return phys->vtable.phys->phys_write_raw_list(phys->instance, write_data, write_data_count);
}

struct PhysicalMemoryMetadata connector_metadata(struct ConnectorInstance *phys) {
    return phys->vtable.phys->metadata(phys->instance);
}

void connector_set_mem_map(struct ConnectorInstance *phys, const struct PhysicalMemoryMapping *mem_maps, uintptr_t mem_maps_count) {
    phys->vtable.phys->set_mem_map(phys->instance, mem_maps, mem_maps_count);
}

void connector_read_raw(struct ConnectorInstance *phys, struct PhysicalAddress addr, uint8_t *out, uintptr_t len) {
    struct PhysicalReadData read_data = {
        .base = addr,
        .size = len;
        Address real_base;
    };
    phys->vtable.phys->set_mem_map(phys->instance, mem_maps, mem_maps_count);
}
*/


// `PhysicalMemory` helpers
/*
int32_t phys_read_raw_list(struct ConnectorInstance *mem, struct PhysicalReadData *data, uintptr_t len) {
    struct PhysicalMemoryInstance phys = {
        .instance = mem->instance,
        .vtable = mem->vtable.phys,
    };
    return phys_read_raw_list(&phys, data, len);
}

int32_t phys_write_raw_list(struct ConnectorInstance *mem, const struct PhysicalWriteData *data, uintptr_t len) {
    struct PhysicalMemoryInstance phys = {
        .instance = mem->instance,
        .vtable = mem->vtable.phys,
    };
    return phys_write_raw_list(&phys, data, len);
}

struct PhysicalMemoryMetadata phys_metadata(struct ConnectorInstance *mem) {
    struct PhysicalMemoryInstance phys = {
        .instance = mem->instance,
        .vtable = mem->vtable.phys,
    };
    return phys_metadata(&phys);
}

void phys_set_mem_map(struct ConnectorInstance *mem, const struct PhysicalMemoryMapping *maps, uintptr_t len) {
    struct PhysicalMemoryInstance phys = {
        .instance = mem->instance,
        .vtable = mem->vtable.phys,
    };
    return phys_set_mem_map(&phys, data, len);
}
*/


// `VirtualMemory` helpers

// `ConnectorInstance` helpers

// `OsInner` helpers

#endif