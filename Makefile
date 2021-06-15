#!/usr/bin/make -f
#CONTAINER_RUNTIME := $(shell which podman 2>/dev/null || shell which docker 2>/dev/null)
CONTAINER_RUNTIME := /usr/local/bin/docker

.PHONY: all
all: fmt lint test schema optimize

.PHONY: develop
develop: test schema optimize

.PHONY: clean
clean:
	@cargo clean

.PHONY: fmt
fmt:
	@cargo fmt --all -- --check

.PHONY: lint
lint:
	@cargo clippy -- -D warnings

.PHONY: build
build:
	@cargo wasm

.PHONY: test
test:
	@cargo test --verbose

.PHONY: schema
schema:
	@cargo run --example schema

.PHONY: optimize
optimize:
	$(CONTAINER_RUNTIME) run --rm -v $(CURDIR):/code:Z \
		--mount type=volume,source=warehouse_facility_cache,target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.10.9
