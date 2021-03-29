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

ENV RUST_LOG httpmock=info

RUN mkdir /mocks

ENTRYPOINT ["httpmock", "--expose", "--static-mock-dir=/mocks"]

EXPOSE 5000