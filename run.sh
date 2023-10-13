#!/bin/bash

if [ ! -d isabelle-gc ] ; then
	git clone https://github.com/isabelle-platform/isabelle-gc.git
	pushd isabelle-gc
	./install.sh
	popd
fi
RUST_LOG=info ./target/debug/isabelle-core --data-path $(pwd)/sample-data --gc-path $(pwd)/isabelle-gc --python-path $(which python3)
