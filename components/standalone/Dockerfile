FROM rust:latest

WORKDIR /usr/src/httpmock
COPY ../.. .

RUN cargo install --path .

EXPOSE 5000

ENV RUST_LOG httpmock=info
CMD ["httpmock", "--expose"]
