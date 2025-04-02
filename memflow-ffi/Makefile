all: build

.PHONY: build
bindgen:
	./scripts/bindgen.sh

.PHONY: check
check:
	./scripts/bindgen_check.sh

.PHONY: examples
examples:
	cd examples/c && make all
	cd examples/cpp && make all
