#!/bin/bash
set -e

echo " 0/10 build"
  cargo build -q

echo " 1/10 test"
  cargo test -q > /dev/null

echo " 2/10 e2e (Interpreted)"
  ./target/debug/stacklang examples/e2e.sl > /dev/null

echo " 3/10 e2e (C)"
  ./target/debug/stacklang -g c examples/e2e.sl > gen/gen.c
  gcc -o gen/out gen/gen.c -Wall -std=c99 -pedantic
  ./gen/out > /dev/null

echo " 4/10 e2e (JS)"
  ./target/debug/stacklang -g js examples/e2e.sl | node > /dev/null

echo " 5/10 e2e (Rust)"
  ./target/debug/stacklang -g rs examples/e2e.sl > gen/gen.rs 
  chmod +x gen/gen.rs
  ./gen/gen.rs > /dev/null 2>/dev/null

echo " 6/10 unicode (Interpreted)"
  ./target/debug/stacklang examples/unicode.sl > /dev/null

echo " 7/10 unicode (JS)"
  ./target/debug/stacklang -g js examples/unicode.sl | node > /dev/null

echo " 8/10 unicode (Rust)"
  ./target/debug/stacklang -g rs examples/unicode.sl > gen/unicode.rs 
  chmod +x gen/unicode.rs
  ./gen/unicode.rs > /dev/null 2>/dev/null

echo " 9/10 lint"
  cargo clippy -q --all-targets
  cargo fmt --check

echo "10/10 success!"
