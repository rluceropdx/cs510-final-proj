FROM rust:buster as builder

WORKDIR /app
RUN USER=root cargo new --bin backend
COPY Cargo.toml Cargo.lock ./backend/

WORKDIR /app/backend
ENV SQLX_OFFLINE=1
RUN cargo build

COPY ./ ./
ENV SQLX_OFFLINE=1
RUN cargo build

FROM debian:buster-slim
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /app

COPY --from=builder /app/backend/target/debug/backend ./
COPY ./templates ./templates
ENV SQLX_OFFLINE=1
CMD ["/app/backend"]