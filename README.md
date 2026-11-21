# coreutils

Rust implementations of `cat`, `grep` and `wc`.

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run --bin cat -- file.txt
cargo run --bin grep -- pattern file.txt
cargo run --bin wc -- file.txt
```

`wc` prints the line, word and byte counts of each file.

Without file arguments, all three read from standard input:

```sh
echo hello | cargo run --bin grep -- hello
```

## Test

```sh
cargo test
```
