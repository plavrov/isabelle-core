#!/bin/bash
TOP_DIR="$(cd "$(dirname "$(which "$0")")" ; pwd -P)"
cd "$TOP_DIR"

port="$1"
pub_url="$2"
data_path="$3"
py_path="$4"
gc_path="$5"

if [ "$port" == "" ] ; then
	port="8090"
fi

if [ "$pub_url" == "" ] ; then
	pub_url="http://localhost:8081"
fi

if [ "$data_path" == "" ] ; then
	data_path="$(pwd)/sample-data"
fi

if [ "$py_path" == "" ] ; then
	if [ "$(uname)" == "Darwin" ] ; then
		py_path="/opt/homebrew/bin/python3"
	else
		py_path="$(which python3)"
	fi
fi

if [ "$gc_path" == "" ] ; then
	if [ ! -d isabelle-gc ] ; then
		git clone https://github.com/isabelle-platform/isabelle-gc.git
		pushd isabelle-gc
		./install.sh
		popd
	fi
fi

RUST_LOG=info ./target/debug/isabelle-core --port "${port}" --pub-url "${pub_url}" --data-path "${data_path}" --gc-path "$(pwd)/isabelle-gc" --py-path "${py_path}"
