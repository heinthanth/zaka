# Two images of zakac, each of which runs zakac itself:
#
#   docker build -t zaka .                   builds zakac from this source tree
#   docker build --target release …          copies in the zakac of a release
#
# The release workflow builds the second for each Linux CPU and pushes it to
# ghcr.io; it unpacks the executable of each CPU to dist/docker/<arch>/zakac
# first, named as Docker names the CPU (amd64, arm64).
#
# Both hold a C compiler and the headers of libc, as the rust image does, so
# that `zakac build --backend c` and `--backend native` work in them. GCC is
# the default; `--build-arg CC_PACKAGES=clang` gives Clang in its place, and
# `--build-arg CC_PACKAGES=` gives an image with no C compiler, for the VM and
# the JIT.

FROM rust:1-slim-bookworm AS build

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin zakac \
    && cp target/release/zakac /usr/local/bin/zakac

FROM debian:bookworm-slim AS base

ARG CC_PACKAGES=gcc
RUN [ -z "$CC_PACKAGES" ] || { apt-get update \
    && apt-get install -y --no-install-recommends $CC_PACKAGES libc6-dev \
    && rm -rf /var/lib/apt/lists/*; }

FROM base AS release

ARG TARGETARCH
COPY dist/docker/${TARGETARCH}/zakac /usr/local/bin/zakac

CMD ["--help"]
ENTRYPOINT ["zakac"]

FROM base AS run

COPY --from=build /usr/local/bin/zakac /usr/local/bin/zakac

CMD ["--help"]
ENTRYPOINT ["zakac"]
