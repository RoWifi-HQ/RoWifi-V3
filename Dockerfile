FROM alpine:latest as builder 
RUN apk upgrade && apk add gcc curl musl-dev wget tar unzip gzip fontconfig-dev make freetype-dev cmake expat-dev
RUN curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y
WORKDIR /usr/src/rowifi
COPY . .
RUN source $HOME/.cargo/env && cargo build --release

FROM alpine:latest
RUN apk upgrade && apk add fontconfig ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]