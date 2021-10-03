CC =gcc
CFLAGS =-I../ -I../../memflow-ffi/ -L../../target/release
LIBS=-Wl,--no-as-needed -lm -ldl -lpthread -pthread -l:libmemflow_win32_ffi.a

ODIR=./

%.o: %.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

phys_mem.out: phys_mem.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

.PHONY: all
all: phys_mem.out

.DEFAULT_GOAL := all

clean:
	rm -f $(ODIR)/*.o
