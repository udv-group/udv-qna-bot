FROM rust:1.73-slim as builder

WORKDIR /build
ENV SQLX_OFFLINE=true

RUN USER=root cargo new --bin bot
RUN USER=root cargo new --bin db
RUN USER=root cargo new --bin server


COPY ./Cargo.toml Cargo.toml
COPY ./bot/Cargo.toml bot/Cargo.toml
COPY ./db/Cargo.toml db/Cargo.toml
COPY ./server/Cargo.toml server/Cargo.toml

RUN apt-get update && apt-get install libssl-dev libsqlite3-dev pkg-config -y

RUN cargo build --release -p server

RUN rm server/src/*.rs
RUN rm db/src/*.rs

COPY ./server/src server/src
COPY ./db/src db/src
COPY ./db/sqlx-data.json db/sqlx-data.json
COPY ./db/migrations db/migrations
COPY ./server/templates server/templates

RUN rm /build/target/release/deps/server*
RUN cargo build --release -p server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install libssl-dev libsqlite3-dev ca-certificates -y
COPY --from=builder /build/target/release/server /app/server

WORKDIR /app
CMD ["/app/server"]
