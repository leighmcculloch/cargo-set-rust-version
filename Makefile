CARGO_DOC_ARGS?=--open
CARGO_TEST_SUBCOMMAND:=$(shell type -p cargo-nextest >/dev/null && echo nextest run || echo test)

export RUSTFLAGS=-Dwarnings -Dclippy::all -Dclippy::pedantic

default: fmt check test readme install

fmt:
	cargo fmt --all --check

check: fmt
	cargo clippy

test: fmt
	cargo $(CARGO_TEST_SUBCOMMAND)

install:
	cargo install --path .

watch:
	cargo watch --clear --watch-when-idle --shell '$(MAKE)'

doc:
	cargo +nightly doc --no-deps --bin cargo-set-rust-version $(CARGO_DOC_ARGS)

readme:
	cargo readme > README.md

publish:
	cargo publish
