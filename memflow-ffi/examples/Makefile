CC =gcc
CFLAGS =-I../ -I../../memflow-ffi/ -L../../target/release
LIBS=-lm -ldl -lpthread -l:libmemflow_win32_ffi.a

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
