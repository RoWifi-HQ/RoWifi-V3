FROM debian:buster as builder 
RUN apt-get update -y && apt-get install -y gcc curl libfontconfig1-dev make libfreetype6-dev cmake libexpat1-dev
RUN curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /usr/src/rowifi
COPY . .
RUN cargo build --release

FROM debian:buster
RUN apt-get update && apt-get install -y libfontconfig1-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rowifi/target/release/rowifi /usr/local/bin/rowifi
CMD ["rowifi"]