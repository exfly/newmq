.PHONY: lint
lint:
	cargo clippy

.PHONY: check
check: 
	cargo check -v

.PHONY: build
build:
	cargo build -v

.PHONY: server
server:
	cargo run --bin server

.PHONY: client
client:
	cargo run --bin client

.PHONY: watch
watch:
	cargo watch -x fmt -x run

.PHONY: test
test: fmt
	cargo test

.PHONY: fmt
fmt: 
	cargo fmt

.PHONY: doc
doc:
	cargo doc

.PHONY: release
release: 
	cargo build --release

.PHONY: ast
ast:
	cargo rustc -- -Z ast-json

.PHONY: macro_expand
macro_expand:
	cargo rustc -- -Z unstable-options --pretty=expanded

.PHONY: dep
dep:
	rustup component add rustfmt
	rustup component add clippy

.PHONY: clean
clean:
	cargo clean
