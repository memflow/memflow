#ifndef MEMFLOW_SUPPORT_H
#define MEMFLOW_SUPPORT_H

#include "memflow.h"

// `PhysicalMemory` helpers
int32_t pmem_read_raw_list(struct PhysicalMemoryInstance *phys, struct PhysicalReadData *read_data, uintptr_t read_data_count) {
    return phys->vtable->phys_read_raw_list(phys->instance, read_data, read_data_count);
}

int32_t pmem_write_raw_list(struct PhysicalMemoryInstance *phys, const struct PhysicalWriteData *write_data, uintptr_t write_data_count) {
    return phys->vtable->phys_write_raw_list(phys->instance, write_data, write_data_count);
}

struct PhysicalMemoryMetadata pmem_metadata(struct PhysicalMemoryInstance *phys) {
    return phys->vtable->metadata(phys->instance);
}

void pmem_set_mem_map(struct PhysicalMemoryInstance *phys, const struct PhysicalMemoryMapping *mem_maps, uintptr_t mem_maps_count) {
    phys->vtable->set_mem_map(phys->instance, mem_maps, mem_maps_count);
}

// phys_read_raw_into
// phys_read_into
// phys_read_raw
// phys_read
// phys_write_raw
// phys_write
// phys_read_ptr32_into
// phys_read_ptr32
// phys_read_ptr64_into
// phys_read_ptr64
// phys_write_ptr32
// phys_write_ptr64
// phys_batcher

// `VirtualMemory` helpers

// `ConnectorInstance` helpers

// `OsInner` helpers

#endif