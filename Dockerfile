FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./kong_rs ./kong_rs
COPY ./kong_rs_macros ./kong_rs_macros
COPY ./kong_rs_protos ./kong_rs_protos
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .

RUN cargo build --release
RUN mv ./target/release/<your-crate> ./app