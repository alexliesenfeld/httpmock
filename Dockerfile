FROM rust:1.40 as builder
WORKDIR /usr/src/httpmock
COPY . .
RUN cargo install --features="standalone" --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/httpmock /usr/local/bin/httpmock

EXPOSE 5000

ENV RUST_LOG httpmock=info
ENTRYPOINT httpmock --expose
