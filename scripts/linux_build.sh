curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo -V

apt install git libdbus-1-dev pkg-config -y

cd ../
cargo build --release

cp target/claw-agent-client-rs   .claw-agent-client-rs
cp scripts/linux_install.sh   install.sh