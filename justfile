list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all

test:
  cargo test --all

clippy:
  cargo clippy --all

bench:
  cargo bench --all

checks:
  just format
  just build
  just clippy
  just test

docs:
  cargo doc --workspace --no-deps

docs-open:
  cargo doc --workspace --no-deps --open

bake-readme:
  cargo readme --project-root ./core/ > README.md

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./derive/Cargo.toml --aggressive
  cargo update --manifest-path ./core/Cargo.toml --aggressive
  cargo update --manifest-path ./tagged/Cargo.toml --aggressive

publish:
  cargo publish --manifest-path ./derive/Cargo.toml --no-verify
  sleep 1
  cargo publish --manifest-path ./core/Cargo.toml --no-verify
  sleep 1
  cargo publish --manifest-path ./tagged/Cargo.toml --no-verify
