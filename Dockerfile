FROM rust:1-alpine as builder
RUN apk update && apk add musl-dev
WORKDIR /src
COPY Cargo.lock Cargo.toml ./
COPY src/ ./src/
RUN cargo build --release

FROM alpine
RUN apk update && apk add lighttpd
COPY lighttpd.conf /etc/lighttpd/lighttpd.conf

WORKDIR /var/www/vent
COPY --from=builder /src/target/release/vent ./target/release/vent
COPY static/ ./static/
COPY template/ ./template/

RUN touch vent.csv && ./target/release/vent render > ./static/vent.html

EXPOSE 80
CMD ["/usr/sbin/lighttpd", "-D", "-f", "/etc/lighttpd/lighttpd.conf"]
