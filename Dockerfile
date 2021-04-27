# ================================================================================
# Builder
# ================================================================================
FROM rust:1.46 as builder
WORKDIR /usr/src/httpmock

COPY Cargo.toml .

COPY src/ ./src/

RUN cargo install --features="standalone" --path .

# ================================================================================
# Runner
# ================================================================================
FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/httpmock /usr/local/bin/httpmock

# Log level (refer to env_logger crate for more information)
ENV RUST_LOG httpmock=info

# The TCP port on which the mock server will listen to.
ENV HTTPMOCK_PORT 5000

# Container internal directory path that contains file bases mock specs (YAML-fies).
# ENV HTTPMOCK_MOCK_FILES_DIR /mocks

# The existance of this environment variable (even if value is empty) is considered "true"/"disabled".
# ENV HTTPMOCK_DISABLE_ACCESS_LOG true

# Request history limit.
ENV HTTPMOCK_REQUEST_HISTORY_LIMIT 100

ENTRYPOINT ["httpmock", "--expose", "true"]

EXPOSE ${HTTPMOCK_PORT}