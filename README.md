# RoWifi - The 2nd Gen Roblox-Discord Verification Bot
Highly customizable bot written in Rust to make your Discord server's integration with your Roblox's group extremely flexible.

WARNING: This repository has been archived. Further development of the bot is being conducted in a private repository.

## What you need to run this
- Rust (nightly channel)
- Docker
- Redis
- MongoDB
- A lot of Linux/Windows libraries

## How to run this
You will find a list of environment variables to set in [here](https://github.com/RoWifi-HQ/RoWifi-V3/blob/master/rowifi/src/main.rs). You will also need the **Guild Members** Intent found on the Discord Developers Dashboard. After this, you are on your own. Be mindful, this is Rust, compile times are ridiculously high.

The [Dockerfile](https://github.com/RoWifi-HQ/RoWifi-V3/blob/master/Dockerfile) is catered to the `aarch64` architecture. If you wish to host it on any other architecture (I assume you're using Docker), make the necessary changes.

If you're running this locally, you can just do
```sh
cargo run # to run a development build
# or
cargo run --release # to run a release build
```

## Disclaimer
This bot is not easy to setup and I do not have the time to make it easier. You should be fine in running this if you manage to install all required underlying packages. You'll find all crates distributed across `Cargo.toml` files or you can find them all under [Cargo.lock](https://github.com/RoWifi-HQ/RoWifi-V3/blob/master/Cargo.lock)

## Contributing
We always welcome meaningful contributions from those who are eager to help. **ALWAYS** make a PR to the `dev` branch. PRs to the `master` branch will be straight up rejected. Also make sure to run `cargo fmt` and `cargo clippy` from the **nightly channel** on the code you submit as a PR. I should not be saying this, but for the love of god, please test your PRs.
