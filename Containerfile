FROM docker.io/ubuntu:24.04

RUN apt-get update -y && \
    apt-get install -y ca-certificates libssl3 libmysqlclient21 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./backend/templates /app/templates
COPY ./backend/target/release/backend /app/backend

EXPOSE 7270
CMD ["/app/backend"]
