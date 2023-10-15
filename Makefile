.PHONY: build
build:
	cargo build

.PHONY: test-local
test-local:
	cargo test

.PHONY: test-remote
test-local:
	cargo test --features=remote

.PHONY: test-standalone
test-standalone:
	cargo test --features standalone

.PHONY: test-all
test-all:
	cargo test --all-features

.PHONY: build-docker
build-docker:
	docker build .