CC = clang
CFLAGS = -g -O0 -I../../../memflow-ffi/ -L../../../target/release
LIBS=-lm -ldl -lpthread -l:libmemflow_ffi.a

ODIR=./

%.o: %.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

phys_mem.out: phys_mem.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

process_list.out: process_list.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

module_list.out: module_list.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

module_dump.out: module_dump.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

find_process.out: find_process.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

.PHONY: all
all: phys_mem.out process_list.out module_list.out module_dump.out find_process.out

.DEFAULT_GOAL := all

clean:
	rm -f $(ODIR)/*.o
