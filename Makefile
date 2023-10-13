.PHONY: build

all: build

build:
	cargo build && ./run.sh
