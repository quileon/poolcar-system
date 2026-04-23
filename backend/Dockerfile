FROM docker.io/clux/muslrust:stable AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release --bin poolcar_backend --bin create_admin

FROM gcr.io/distroless/static-debian12 AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/poolcar_backend /poolcar_backend
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/create_admin /create_admin
CMD ["/poolcar_backend"]
