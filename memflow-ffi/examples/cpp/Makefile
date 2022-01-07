CC = g++
CFLAGS = -g -O0 -pedantic -std=c++14 -I../../../memflow-ffi/ -L../../../target/release
LIBS=-lm -ldl -lpthread -l:libmemflow_ffi.a

ODIR=./

%.o: %.cpp $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

plist.out: plist.o
	$(CC) -o $@ $^ $(CFLAGS) $(LIBS)

.PHONY: all
all: plist.out

.DEFAULT_GOAL := all

clean:
	rm -f $(ODIR)/*.o
