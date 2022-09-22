FROM rust:1.62 AS builder
COPY . .
RUN cargo build --release

FROM debian:bullseye
COPY --from=builder ./target/release/insert_benchmark ./target/release/insert_benchmark

CMD ["/target/release/insert_benchmark"]
