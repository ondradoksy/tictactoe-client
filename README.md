# Tic-tac-toe game web client

The web client for a Tic-tac-toe game.

[![Rust](https://github.com/ondradoksy/tictactoe-client/actions/workflows/rust.yml/badge.svg)](https://github.com/ondradoksy/tictactoe-client/actions/workflows/rust.yml)

# Building

Install [rustup](https://doc.rust-lang.org/cargo/getting-started/installation.html) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

Build using:

```
$ wasm-pack build
```

## Using the watch script

Install [cargo-watch](https://github.com/watchexec/cargo-watch)

Run watch script

```
$ ./watch.sh
```

## Running the web server

Install [node](https://github.com/nodejs/node) and [npm](https://github.com/npm/cli)

Install dependencies (run inside `www` directory)

```
$ npm install
```

Run the web server

```
$ www/run.sh
```

Web server will start and listen on localhost:8080
