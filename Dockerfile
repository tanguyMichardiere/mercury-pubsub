FROM rust as rust-builder
ARG DATABASE_URL

WORKDIR /usr/src/
RUN apt-get update && apt-get install -y musl-tools
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
RUN rustup target add x86_64-unknown-linux-musl

RUN cargo new app
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/main.rs

COPY src src
COPY sqlx-data.json .
RUN RUSTFLAGS="-C target-cpu=native" SQLX_OFFLINE=true cargo build --release --features=secure --bin=main --package=server --target x86_64-unknown-linux-musl
COPY migrations migrations
RUN sqlx migrate run

FROM node:16 as dashboard-builder

WORKDIR /usr/src/app

COPY dashboard/package.json dashboard/package-lock.json ./
RUN npm clean-install

COPY dashboard .
RUN npm run build

FROM scratch

COPY --from=rust-builder /usr/src/app/target/x86_64-unknown-linux-musl/release/main /server
COPY --from=dashboard-builder /usr/src/app/out /static
CMD ["/server"]
