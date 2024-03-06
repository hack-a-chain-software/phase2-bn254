# Use ubuntu 20.04 for building, otherwise the binary will not work on gramine image
cd ./phase2

cargo build --release --bin new
cargo build --release --bin contribute
cargo build --release --bin verify_contribution

mkdir -p ../dist/bin
cp target/release/new ../../gramine/dist/bin
cp target/release/contribute ../../gramine/dist/bin
cp target/release/verify_contribution ../../gramine/dist/bin