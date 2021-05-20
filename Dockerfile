FROM debian:buster as builder 
RUN apt-get update -y && apt-get install curl wget lsb-release software-properties-common gnupg -y
RUN curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y
RUN wget https://github.com/Kitware/CMake/releases/download/v3.20.2/cmake-3.20.2-Linux-aarch64.sh \
      -q -O /tmp/cmake-install.sh \
      && chmod u+x /tmp/cmake-install.sh \
      && mkdir /usr/bin/cmake \
      && /tmp/cmake-install.sh --skip-license --prefix=/usr/bin/cmake \
      && rm /tmp/cmake-install.sh
ENV PATH="/usr/bin/cmake/bin:${PATH}"
RUN bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"
WORKDIR /usr/src/rowifi
COPY . .
RUN source $HOME/.cargo/env && cargo build --release

FROM debian:buster
RUN apt-get update -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]