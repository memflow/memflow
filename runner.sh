#!/bin/bash

if [[ "$@" =~ "qemu_procfs" ]]; then
	CWD=$(pwd)
	DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
	cd $DIR
	bash setperms.sh
	cd $CWD
fi

exec $@
