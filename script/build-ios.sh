# cargo install cargo-lipo

cd `dirname $0`/../localnative-rs/localnative_core

# echo 'crate-type = ["staticlib"]' >> Cargo.toml

# cargo clean

cargo lipo --release

# git reset --hard
