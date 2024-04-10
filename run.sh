#!/bin/bash
TOP_DIR="$(cd "$(dirname "$(which "$0")")" ; pwd -P)"
cd "$TOP_DIR"

binary="${BINARY:-./target/debug/isabelle-core}"
if [ ! -f "${binary}" ] ; then
    binary="./isabelle-core"
fi

first_run="${FIRST_RUN:+--first-run}"
port="8090"
pub_url="http://localhost:8081"
pub_fqdn="localhost"
data_path="$(pwd)/data-equestrian"
py_path=""
gc_path="$6"
database="isabelle"
gh_login=""
gh_password=""

if [ "$(uname)" == "Darwin" ] ; then
    py_path="/opt/homebrew/bin/python3"
else
    py_path="$(which python3)"
fi

while test -n "$1" ; do
    case "$1" in
        --port)
            port="$2"
            shift 1
            ;;
        --pub-url)
            pub_url="$2"
            shift 1
            ;;
        --pub-fqdn)
            pub_fqdn="$2"
            shift 1
            ;;
        --data-path)
            data_path="$2"
            shift 1
            ;;
        --py-path)
            py_path="$2"
            shift 1
            ;;
        --gc-path)
            gc_path="$2"
            shift 1
            ;;
        --first-run)
            first_run="--first-run"
            ;;
        --database)
            database="$2"
            shift 1
            ;;
        --gh-login)
            gh_login="$2"
            shift 1
            ;;
        --gh-password)
            gh_password="$2"
            shift 1
            ;;
    esac
    shift 1
done

if [ "$gc_path" == "" ] ; then
    if [ ! -d isabelle-gc ] ; then
        creds=""
        if [ "$gh_login" != "" ] && [ "$gh_password" != "" ] ; then
            creds="${gh_login}:${gh_password}@"
        fi
        git clone https://${creds}github.com/isabelle-platform/isabelle-gc.git
        pushd isabelle-gc
        ./install.sh
        popd
    fi
    gc_path="$(pwd)/isabelle-gc"
fi

if [ "$(uname)" == "Darwin" ] ; then
    /usr/libexec/PlistBuddy -c "Add :com.apple.security.get-task-allow bool true" tmp.entitlements
    for file in ${binary} $(ls libisabelle_plugin*) ; do
        codesign -s - -f --entitlements tmp.entitlements "$file"
    done
fi

RUST_LOG=info RUST_BACKTRACE=1 "${binary}" --port "${port}" --pub-url "${pub_url}" --pub-fqdn "${pub_fqdn}" --data-path "${data_path}" --gc-path "${gc_path}" --database "${database}" --py-path "${py_path}" ${first_run}
