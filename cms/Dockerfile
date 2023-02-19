FROM rust:1.67-slim as builder

WORKDIR /build
ENV SQLX_OFFLINE=true

RUN USER=root cargo new --bin bot
RUN USER=root cargo new --bin cms
RUN USER=root cargo new --bin db

COPY ./Cargo.toml Cargo.toml
COPY ./Cargo.lock Cargo.lock
COPY ./bot/Cargo.toml bot/Cargo.toml
COPY ./db/Cargo.toml db/Cargo.toml
COPY ./cms/Cargo.toml cms/Cargo.toml

RUN apt-get update && apt-get install libssl-dev libsqlite3-dev pkg-config -y

RUN cargo build --release -p cms

RUN rm cms/src/*.rs
RUN rm db/src/*.rs

COPY ./cms/src cms/src
COPY ./db/src db/src
COPY ./db/sqlx-data.json db/sqlx-data.json
COPY ./db/migrations db/migrations

RUN rm /build/target/release/deps/cms*
RUN cargo build --release -p cms

FROM debian:bullseye-slim
RUN apt-get update && apt-get install libssl-dev libsqlite3-dev ca-certificates -y
COPY --from=builder /build/target/release/cms /app/cms
COPY ./cms/Rocket.toml /app/Rocket.toml
COPY ./cms/templates /app/templates

WORKDIR /app
CMD ["/app/cms"]