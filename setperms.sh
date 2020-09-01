#!/bin/bash

do_setcap() {
	for f in "$1/$2"*; do
		if [[ -f $f && $f != *.* ]] ; then
			if [[ -z "$(getcap $f | grep -i cap_sys_ptrace)" ]]; then
				echo setcap for $f
				sudo setcap 'CAP_SYS_PTRACE=ep' $f
			fi
		fi
	done
}

files=(
    examples/read_bench
    examples/integration
    examples/read_keys
    examples/dump_offsets
    deps/read_win32
)

for f in ${files[*]}; do
	do_setcap target/debug $f;
done

for f in ${files[*]}; do
	do_setcap target/release $f;
done
