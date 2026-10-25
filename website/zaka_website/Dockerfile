# syntax=docker/dockerfile:1

FROM rust:1-slim-bookworm AS build

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --locked trunk@0.22.0 cargo-leptos@0.3.11

WORKDIR /src
COPY . .

# The address the site is served from, which the cover card and the other
# links that must stand alone start with, on the website and the explorer:
# pass it with --build-arg.
ARG ZAKA_SITE_ORIGIN

# The website writes its pages once and stops, so the site is plain files.
# The motion libraries, the web fonts and the explorer join it under /vendor,
# /fonts and /explorer, placed as `mise run web:build` places them.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/root/.cache \
    --mount=type=secret,id=zaka_pda_font_secret,env=ZAKA_PDA_FONT_SECRET \
    (cd website/zaka_website/vendor && trunk build --release) \
    && (cd website/zaka_web_ui_common/fonts && trunk build --release) \
    && cargo leptos build --project zaka_website --release \
    && LEPTOS_OUTPUT_NAME=zaka_website LEPTOS_SITE_ROOT=target/website \
        LEPTOS_SITE_PKG_DIR=pkg LEPTOS_HASH_FILES=true ZAKA_WEBSITE_ONCE=1 \
        target/release/zaka_website \
    && (cd website/zaka_explorer \
        && trunk build --release --locked --cargo-profile wasm-release \
            --public-url /explorer/) \
    && mkdir -p /srv/site \
    && cp -R target/website/. /srv/site/ \
    && cp -R target/vendor /srv/site/vendor \
    && cp -R target/fonts /srv/site/fonts \
    && cp -R website/zaka_explorer/dist /srv/site/explorer

FROM joseluisq/static-web-server:2 AS run

COPY --from=build /srv/site /public
