FROM rust:latest as builder
WORKDIR /usr/src/rowifi
RUN wget https://github.com/Kitware/CMake/releases/download/v3.19.6/cmake-3.19.6-Linux-aarch64.sh \
      -q -O /tmp/cmake-install.sh \
      && chmod u+x /tmp/cmake-install.sh \
      && mkdir /usr/bin/cmake \
      && /tmp/cmake-install.sh --skip-license --prefix=/usr/bin/cmake \
      && rm /tmp/cmake-install.sh
ENV PATH="/usr/bin/cmake/bin:${PATH}"
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libfontconfig libfontconfig1-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]