FROM rust:1.67-alpine

WORKDIR /usr/src/backend

RUN apk add --no-cache cargo-watch musl-dev protoc && \
  rustup component add rustfmt

COPY backend/Cargo.toml ./Cargo.toml
COPY backend/Cargo.lock* ./
COPY backend/src ./src
