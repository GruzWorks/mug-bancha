# mug-bancha
Find publicly accessible power sockets in your city! Backend server

**This project is discontinued.**

## Building from source
### Requirements
* Rust toolchain (preferably stable) supporting the 2018 edition

### Build
Clone the repository and `cd` into it, then run `cargo build`.

```sh
$ git clone https://github.com/baatochan/mug-bancha.git
$ cd mug-bancha
$ cargo build
```

### Test
Run tests with `cargo test -- --test-threads 1`. Parallel test execution is temporarily not supported because of the way the in-memory storage is initialised. A solution is currently in progress.

### Run
Run with `cargo run`. The server listens on `127.0.0.1:3000` by default.
