FROM ekidd/rust-musl-builder:nightly-2021-12-23 as builder

ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:latest

COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/oil \
    /usr/local/bin/

EXPOSE 80
CMD ["/usr/local/bin/oil"]