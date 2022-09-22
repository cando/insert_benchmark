FROM rust:1.62 AS builder
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder ./target/release/insert_benchmark ./target/release/insert_benchmark

RUN apt-get update 
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends pkg-config libssl-dev -y

RUN ln -s /usr/lib/x86_64-linux-gnu/libssl.so.1.1 /usr/lib/x86_64-linux-gnu/libssl.so.3 && \
    ln -s /usr/lib/x86_64-linux-gnu/libcrypto.so.1.1 /usr/lib/x86_64-linux-gnu/libcrypto.so.3 && \
    ldconfig

CMD ["/target/release/insert_benchmark"]
