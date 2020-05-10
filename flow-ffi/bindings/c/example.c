#include <stdio.h>
#include "memflow.h"

int main(int argc, char *argv[]) {
    printf("initializing memflow...\n");

    log_init(LOG_DEBUG);

    void *mem = qemu_procfs_init();
    if (mem == 0) {
        printf("qemu_procfs backend could not be initialized\n");
        return 1;
    }

    void *win32 = win32_init(mem);
    if (win32 == 0) {
        printf("win32 failed to initialize\n");
        qemu_procfs_free(mem);
        return 1;
    }

    void *offsets = win32_offsets_init(win32);
    if (offsets == 0) {
        printf("win32 offsets failed to inizialize\n");
        win32_free(win32);
        qemu_procfs_free(mem);
        return 1;
    }

    printf("memflow initialized\n");

    win32_offsets_free(offsets);
    win32_free(win32);
    qemu_procfs_free(mem);
    printf("memflow freed\n");

    return 0;
}
