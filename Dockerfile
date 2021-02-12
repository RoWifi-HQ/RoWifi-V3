FROM rustlang/rust@sha256:319fd4f46b09e5b426b037181a6c0e5a160f1af7c5e316b5b89d06023d2b8d2e as builder
WORKDIR /usr/src/rowifi
RUN wget https://github.com/Kitware/CMake/releases/download/v3.18.2/cmake-3.18.2-Linux-x86_64.sh \
      -q -O /tmp/cmake-install.sh \
      && chmod u+x /tmp/cmake-install.sh \
      && mkdir /usr/bin/cmake \
      && /tmp/cmake-install.sh --skip-license --prefix=/usr/bin/cmake \
      && rm /tmp/cmake-install.sh
ENV PATH="/usr/bin/cmake/bin:${PATH}"
RUN echo 'deb http://apt.llvm.org/buster/ llvm-toolchain-buster main' > /etc/apt/sources.list.d/llvm.list && \
    (wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -) && \
    apt-get update && apt-get install -y lld-12 && \
    rm -rf /var/lib/apt/lists/* && \
    ln -s /usr/bin/ld.lld-* /usr/bin/ld.lld
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libfontconfig libfontconfig1-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]