FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./kong_rs ./kong_rs
COPY ./kong_rs_macros ./kong_rs_macros
COPY ./kong_rs_protos ./kong_rs_protos
COPY ./examples ./examples
RUN cargo chef prepare

FROM chef AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .

RUN cargo build --release
RUN mv ./target/release/log ./log

FROM kong:ubuntu as runner
COPY --from=builder /app/log /usr/local/bin/kong-rs-log
COPY ./examples/kong.conf /etc/kong/kong.conf
COPY ./examples/kong.yml /etc/kong/kong.yml