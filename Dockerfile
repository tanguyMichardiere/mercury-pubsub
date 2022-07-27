FROM rust as rust-builder

WORKDIR /usr/src/
RUN apt-get update && RUN apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

RUN cargo new app
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/main.rs

COPY src src
RUN cargo build --release --bin=main --package=server --target x86_64-unknown-linux-musl

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
