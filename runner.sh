#!/bin/bash

echo $@

PCMD=$(cat /proc/$PPID/cmdline | strings -1)
PCMD=$(echo -n ${PCMD##*/})
RNAME=$(echo -n ${@##*/})

if [[ "$PCMD" =~ "cargo test" ]]; then
	exec $@
elif [[ "$PCMD" =~ "cargo bench" ]]; then
	if [[ $RNAME =~ "dummy" ]]; then
		exec $@
	else
		exec sudo $@
	fi
else
	exec sudo $@
fi
