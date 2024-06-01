# isabelle-core

[![Build Status](https://jenkins.interpretica.io/buildStatus/icon?job=isabelle-core%2Fmain)](https://jenkins.interpretica.io/job/isabelle-core/job/main/)

Isabelle is a Rust-based framework for building safe and performant servers for the variety of use cases.

## Features

 - Unified item storage with addition, editing and deletion support.
 - Collection hooks allowing plugins to do additional checks or synchronization.
 - Security checks.
 - E-Mail sending support.
 - Google Calendar integration.
 - Login/logout functionality.
 - One-time password support.

## Dependencies

 - Python 3 is needed for Google Calendar integration

## Building

Building Isabelle is as easy as Cargo invocation:
```
cargo build
```

## Running

Use `run.sh` script:
```
./run.sh
```

## License
MIT
