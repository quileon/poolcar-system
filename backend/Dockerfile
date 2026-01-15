FROM docker.io/clux/muslrust:stable AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/static-debian12 AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/poolcar_backend /
CMD ["./poolcar_backend"]
