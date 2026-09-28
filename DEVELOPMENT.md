# Development

## Test

```bash
cargo check
cargo clippy
cargo test
```

## Check Coverage

`cargo llvm-cov --html`

## Testing Generated Code

```bash
mkdir -p gen
cargo run -- examples/e2e.sl -g js | node
cargo run -- examples/e2e.sl -g rs > gen/e2e.rs && chmod +x gen/e2e.rs && gen/e2e.rs
cargo run -- examples/e2e.sl -g c > gen/e2e.c && gcc -o gen/e2e-c gen/e2e.c && gen/e2e-c

```

