#!/bin/sh

echo -e "\nInstalling required update and upgrade for Linux Debian\n"
read -p "Press enter to continue..."
echo -e "\n"

# Update and Upgrade Linux Debian
sudo apt update && sudo apt upgrade -y
sudo apt install pkg-config libssl-dev -y

echo -e "\nInstalling Starship\n"
read -p "Press enter to continue..."
echo -e "\n"

curl -sS https://starship.rs/install.sh | sh -s -- -y && echo 'eval "$(starship init bash)"' >> ~/.bashrc && echo 'eval "$(starship init zsh)"' >> ~/.zshrc

echo -e "\n"
echo "Done installing Starship"

echo -e "\nDone Updating Linux Debian and Updating RustUp Stable version\n"
read -p "Press enter to continue..."
echo -e "\n"

export CARGO_TARGET_DIR=/tmp/cargo-installtraodP

rustup update stable

echo -e "\nDone Updating RustUp Stable version\n"
read -p "Press enter to continue..."
echo -e "\n"

rustc --version
rustup --version
rustfmt --version

echo -e "\nDone Checking Rust Version\n"
read -p "Press enter to continue..."
echo -e "\n"

cargo --version
cargo --list

echo -e "\nDone Checking Cargo version and list\n"
read -p "Press enter to continue..."
echo -e "\n"

rustup component list

echo -e "\nChecking Rustup Components\n"
read -p "Press enter to continue..."
echo -e "\n"

# Init Rust Project
FILE_CARGO="Cargo.toml"
MAIN_DIR="/src"

# Init Rust Project if Cargo.toml and src/main.rs not exists
if [[ ! -f "$FILE_CARGO" && ! -d "$MAIN_DIR" ]]; then
  cargo init
fi

# Add dependencies
# e.g. cargo add <package_name>

# Core dependencies
cargo add anyhow@1.0.98
cargo add reqwest@0.12.19 --features json
cargo add rmcp@0.1.5 --features server
cargo add serde@1.0.219 --features derive
cargo add serde_json@1.0.140
cargo add tokio@1.45.1 --features macros,rt-multi-thread
cargo add tracing@0.1.41
cargo add tracing-subscriber@0.3.19 --features env-filter

# Web framework dependencies
cargo add axum@0.7
cargo add tower@0.4
cargo add tower-http@0.5 --features cors
cargo add hyper@1.0