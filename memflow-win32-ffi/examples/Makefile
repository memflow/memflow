CC =gcc
CFLAGS =-I../ -I../../memflow-ffi/ -L../../target/release
LIBS=-lm -ldl -lpthread -l:libmemflow_win32_ffi.a

ODIR=./

%.o: %.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

process_list.out: process_list.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

dump_header.out: dump_header.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

.PHONY: all
all: process_list.out dump_header.out

.DEFAULT_GOAL := all

clean:
	rm -f $(ODIR)/*.o
