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

```
RUST_LOG=info ./target/debug/isabelle-core --port 8090 --pub-url http://localhost:8081 --data-path sample-data --gc-path isabelle-gc --py-path $(which python3)
```

## License
MIT
