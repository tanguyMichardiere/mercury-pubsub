FROM rust as builder

ARG DATABASE_URL
ARG TARGET="x86_64-unknown-linux-musl"

ENV RUSTFLAGS="-C target-cpu=native"
ENV TARGET_CC="/usr/bin/clang"
ENV TARGET_AR="/usr/bin/llvm-ar"

WORKDIR /usr/src/
RUN apt-get update && apt-get install -y clang llvm
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
RUN rustup target add $TARGET

RUN cargo new app
WORKDIR /usr/src/app
COPY server/Cargo.toml server/Cargo.lock ./
RUN cargo build --release --target $TARGET
RUN rm src/main.rs

COPY server/src src
COPY server/sqlx-data.json .
RUN SQLX_OFFLINE=true cargo build --release --bin=main --package=server --target $TARGET
COPY server/migrations migrations
RUN sqlx migrate run

FROM scratch

COPY --from=builder /usr/src/app/target/$TARGET/release/main /server
CMD ["/server"]
