#!/bin/bash

do_setcap() {
	for f in "$1/$2"*; do
		if [[ -f $f && $f != *.* ]] ; then
			setcap 'CAP_SYS_PTRACE=ep' $f
		fi
	done
}

files=(examples/read_bench
		examples/read_keys
		deps/read_win32
)

for f in ${files[*]}; do
	do_setcap target/debug $f;
done

for f in ${files[*]}; do
	do_setcap target/release $f;
done
