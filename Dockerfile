FROM rust:1.70.0 AS builder

RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN rustup target add x86_64-unknown-linux-musl
RUN update-ca-certificates

WORKDIR /poc_rear/

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine

WORKDIR /app

COPY --from=builder /poc_rear/target/x86_64-unknown-linux-musl/release/poc_rear ./

ENV SERVICE_PORT=8080 SERVICE_IP=0.0.0.0

CMD ["/app/poc_rear"]
