#!/bin/bash
TOP_DIR="$(cd "$(dirname "$(which "$0")")" ; pwd -P)"
cd "$TOP_DIR"

binary="${BINARY:-./target/debug/isabelle-core}"
first_run="${FIRST_RUN:+--first-run}"
port="$1"
pub_url="$2"
pub_fqdn="$3"
data_path="$4"
py_path="$5"
gc_path="$6"

if [ "$port" == "" ] ; then
	port="8090"
fi

if [ "$pub_url" == "" ] ; then
	pub_url="http://localhost:8081"
fi

if [ "$pub_fqdn" == "" ] ; then
	pub_fqdn="localhost"
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
	gc_path="$(pwd)/isabelle-gc"
fi

RUST_LOG=info RUST_BACKTRACE=1 "${binary}" --port "${port}" --pub-url "${pub_url}" --pub-fqdn "${pub_fqdn}" --data-path "${data_path}" --gc-path "${gc_path}" --py-path "${py_path}" ${first_run}
