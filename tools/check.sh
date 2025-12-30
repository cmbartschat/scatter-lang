#!/bin/bash
set -e

echo " 0/10 build"
  cargo build -q
  
check_count="10"

echo " 1/$check_count test"
  cargo test -q > /dev/null

echo " 2/$check_count e2e (Interpreted)"
  ./target/debug/stacklang examples/e2e.sl > /dev/null

echo " 3/$check_count e2e (C)"
  ./target/debug/stacklang -g c examples/e2e.sl > gen/gen.c
  gcc -o gen/out gen/gen.c -Wall -std=c99 -pedantic
  ./gen/out > /dev/null

echo " 4/$check_count e2e (JS)"
  ./target/debug/stacklang -g js examples/e2e.sl | node > /dev/null

echo " 5/$check_count e2e (Rust)"
  ./target/debug/stacklang -g rs examples/e2e.sl > gen/gen.rs 
  chmod +x gen/gen.rs
  ./gen/gen.rs > /dev/null 2>/dev/null

echo " 6/$check_count unicode (Interpreted)"
  ./target/debug/stacklang examples/unicode.sl > /dev/null

echo " 7/$check_count unicode (JS)"
  ./target/debug/stacklang -g js examples/unicode.sl | node > /dev/null

echo " 8/$check_count unicode (Rust)"
  ./target/debug/stacklang -g rs examples/unicode.sl > gen/unicode.rs 
  chmod +x gen/unicode.rs
  ./gen/unicode.rs > /dev/null 2>/dev/null

echo "9/$check_count lint"
  cargo clippy -q --all-targets
  cargo fmt --check

echo "10/$check_count success!"
