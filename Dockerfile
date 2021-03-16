FROM amazonlinux:latest as builder 
RUN yum update && yum install -y gcc curl wget tar unzip gzip fontconfig && yum group install "Development Tools" -y
RUN curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y
WORKDIR /usr/src/rowifi
RUN wget https://github.com/Kitware/CMake/releases/download/v3.19.6/cmake-3.19.6-Linux-aarch64.sh \
      -q -O /tmp/cmake-install.sh \
      && chmod u+x /tmp/cmake-install.sh \
      && mkdir /usr/bin/cmake \
      && /tmp/cmake-install.sh --skip-license --prefix=/usr/bin/cmake \
      && rm /tmp/cmake-install.sh
ENV PATH="/usr/bin/cmake/bin:${PATH}"
COPY . .
RUN source $HOME/.cargo/env && cargo build --release

FROM amazonlinux:latest
RUN yum update && yum install -y fontconfig ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]