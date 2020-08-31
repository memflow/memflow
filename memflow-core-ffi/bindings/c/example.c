#include <stdio.h>

#include "memflow/memflow.h"
#include "memflow/connectors/coredump.h"

int main(int argc, char *argv[]) {
    printf("initializing memflow...\n");

    if (argc < 2) {
        printf("usage: %s [filename]", argv[0]);
        return 1;
    }

    log_init(LOG_DEBUG);

    void *mem = coredump_open(argv[1]);
    if (mem == 0) {
        printf("qemu_procfs backend could not be initialized\n");
        return 1;
    }

    void *win32 = win32_init(mem);
    if (win32 == 0) {
        printf("win32 failed to initialize\n");
        coredump_free(mem);
        return 1;
    }

    void *offsets = win32_offsets_init(win32);
    if (offsets == 0) {
        printf("win32 offsets failed to inizialize\n");
        win32_free(win32);
        coredump_free(mem);
        return 1;
    }

    printf("memflow initialized\n");

    win32_offsets_free(offsets);
    win32_free(win32);
    coredump_free(mem);
    printf("memflow freed\n");

    return 0;
}
