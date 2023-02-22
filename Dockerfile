FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11:nonroot

WORKDIR /app
COPY --from=builder /app/target/release/members-db /app/members-db

ENV PORT 8080

ENTRYPOINT [ "/app/members-db" ]