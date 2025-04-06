# syntax=docker/dockerfile:1.4

FROM rust:1.86-alpine AS builder

WORKDIR /app

# Cache dependencies
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo fetch --locked

# Copy source code
COPY src src

# Build the application
RUN cargo build --release --frozen


FROM alpine:3.19

RUN apk add --no-cache coreutils

WORKDIR /app

COPY --from=builder /app/target/release/codecrafters-shell /app/codecrafters-shell

ENV TERM xterm-256color
ENV LANG C.UTF-8

ENTRYPOINT ["/app/codecrafters-shell"]
