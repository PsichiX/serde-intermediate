list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all

test:
  cargo test --all --features derive

clippy:
  cargo clippy --all

checks:
  just build
  just test
  just clippy

list-outdated:
  cargo outdated -R -w

update:
  cargo update --workspace

publish:
  cargo publish --manifest-path ./core/Cargo.toml --no-verify
  sleep 15
  cargo publish --manifest-path ./tagged/Cargo.toml --no-verify
  sleep 15
  cargo publish --manifest-path ./reflect-derive/Cargo.toml --no-verify
  sleep 15
  cargo publish --manifest-path ./reflect/Cargo.toml --no-verify
