#include <stdio.h>
#include "memflow.h"

int main(int argc, char *argv[]) {
    printf("trying to connect\n");
    void *ctx = bridge_connect("tcp:127.0.0.1:8181,nodelay");
    if (ctx != 0) {
        printf("connected\n");
        void *win = win32_init_bridge(ctx);
        if (win != 0) {
            printf("win32 initialized\n");
        } else {
            printf("win32 failed to initialize\n");
        }
        bridge_free(ctx);
        printf("disconnected\n");
    } else {
        printf("connection failed\n");
    }
    return 0;
}