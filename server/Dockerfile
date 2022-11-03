ARG TARGET="x86_64-unknown-linux-musl"

FROM rust as builder
ARG TARGET

ENV RUSTFLAGS="-C target-cpu=native"
ENV TARGET_CC="/usr/bin/clang"
ENV TARGET_AR="/usr/bin/llvm-ar"

RUN apt-get update && apt-get install -y clang llvm
RUN rustup target add $TARGET

RUN cargo new server
WORKDIR /server
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target $TARGET
RUN rm src/main.rs

COPY src src
COPY sqlx-data.json .
RUN SQLX_OFFLINE=true cargo build --release --bin=server --package=server --target=$TARGET

FROM scratch
ARG TARGET

COPY --from=builder /server/target/$TARGET/release/server /server
CMD ["/server"]
