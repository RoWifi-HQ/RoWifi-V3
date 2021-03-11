FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/rowifi
RUN yum groupinstall "Development Tools" && yum install cmake
RUN echo 'deb http://apt.llvm.org/buster/ llvm-toolchain-buster main' > /etc/apt/sources.list.d/llvm.list && \
    (wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -) && \
    apt-get update && apt-get install -y lld-13 && \
    rm -rf /var/lib/apt/lists/* && \
    ln -s /usr/bin/ld.lld-* /usr/bin/ld.lld
COPY . .
RUN cargo build --release

FROM amazonlinux:latest
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]