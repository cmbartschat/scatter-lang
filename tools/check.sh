#!/bin/bash
set -e

echo "...Building"
cargo build -q

echo "...Testing"
cargo test -q > /dev/null

echo "...Running e2e"
./target/debug/stacklang -g c examples/e2e.sl > gen/gen.c
gcc -o gen/out gen/gen.c -Wall -std=c99 -pedantic
./gen/out > /dev/null
./target/debug/stacklang -g js examples/e2e.sl | node > /dev/null
./target/debug/stacklang examples/e2e.sl > /dev/null

echo "...Linting"
cargo clippy -q
cargo fmt --check -q

echo "Success!"
