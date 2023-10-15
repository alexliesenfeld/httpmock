.PHONY: build
build:
	cargo build

.PHONY: test-local
test-local:
	cargo test

.PHONY: test-remote
test-local:
	cargo test --features=remote

.PHONY: build-docker
build-docker:
	docker build -t alexliesenfeld/httpmock:latest .