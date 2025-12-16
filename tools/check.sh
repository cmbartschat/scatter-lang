#!/bin/bash
set -e

echo "...Building"
cargo build -q

echo "...Testing"
cargo test -q > /dev/null

echo "...Running e2e"
echo "......Interpreted"
./target/debug/stacklang examples/e2e.sl > /dev/null

echo "......C"
./target/debug/stacklang -g c examples/e2e.sl > gen/gen.c
gcc -o gen/out gen/gen.c -Wall -std=c99 -pedantic
./gen/out > /dev/null

echo "......JS"
./target/debug/stacklang -g js examples/e2e.sl | node > /dev/null

echo "......Rust"
./target/debug/stacklang -g rs examples/e2e.sl > gen/gen.rs 
chmod +x gen/gen.rs
./gen/gen.rs > /dev/null 2>/dev/null

echo "...Linting"
cargo clippy -q
cargo fmt --check -q

echo "Success!"
