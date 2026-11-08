FROM rust:1.99.1-bookworm

RUN rustup component add clippy && useradd --uid 10001 --create-home runner

# Populate this directory only from reviewed manifests/lockfiles, never PR code.
# A derived image can RUN cargo fetch --locked against an approved manifest.
RUN mkdir -p /target /submission && chmod -R a+rX /usr/local/cargo /usr/local/rustup

ENV RUSTUP_TOOLCHAIN=1.99.1 CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/target
USER 10001:10001

WORKDIR /submission

ENTRYPOINT ["cargo"]
