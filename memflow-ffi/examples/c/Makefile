CC = gcc
CFLAGS = -g -O0 -I../../../memflow-ffi/ -L../../../target/release
LIBS=-lm -ldl -lpthread -l:libmemflow_ffi.a

ODIR=./

%.o: %.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

phys_mem.out: phys_mem.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

plist.out: plist.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

.PHONY: all
all: phys_mem.out plist.out

.DEFAULT_GOAL := all

clean:
	rm -f $(ODIR)/*.o
