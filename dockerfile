## Build Stage
FROM rust:alpine3.20 as builder 

RUN apk update && apk add --no-cache musl-dev libressl-dev 

WORKDIR /auth-service
COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release

## Main Stage 
FROM alpine:3.20.1

COPY --from=builder /auth-service/target/release/auth-service . 
COPY appsettings.json .
RUN chmod +x auth-service

CMD ["./auth-service"]