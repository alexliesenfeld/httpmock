FROM rust:latest

WORKDIR /usr/src/httpmock
COPY . .

RUN cargo install --features="standalone" --path .

EXPOSE 5000

ENV RUST_LOG httpmock=info

ENTRYPOINT httpmock --expose
