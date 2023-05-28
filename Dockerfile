####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER=user
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /user

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release
WORKDIR /user/target/x86_64-unknown-linux-musl/release/
RUN mv tg-ycc-bot app && strip -s app

####################################################################################################
## Final image
####################################################################################################
FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /user

COPY --from=builder /user/target/x86_64-unknown-linux-musl/release/app ./
USER user:user

EXPOSE 53899
CMD ["/user/app"]
