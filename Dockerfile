FROM rust as rust-builder

WORKDIR /usr/src/
RUN rustup target add x86_64-unknown-linux-musl

RUN cargo new app
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM node:16 as dashboard-builder

WORKDIR /usr/src/app

COPY dashboard/package.json dashboard/package-lock.json ./
RUN npm clean-install

COPY dashboard .
RUN npm run build

FROM scratch

COPY --from=rust-builder /usr/local/cargo/bin/server .
COPY --from=dashboard-builder /usr/src/app/out static
CMD ["./server"]
