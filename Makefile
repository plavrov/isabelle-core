.PHONY: build

all: build

build:
	cargo build && RUST_LOG=info ./target/debug/isabelle-core
