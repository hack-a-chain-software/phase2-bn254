#!/bin/sh

set -e

if [ ! -f ../powersoftau/phase1radix2m0 ]; then
    echo "Please run powers of tau test first to generate radix files"
    exit 1
fi

# move results of powers of tau here
cp ../powersoftau/phase1radix* .

npm install

# compile circuit

circom ./circuits/circuit_transaction_10x2.circom -o ./circuits/circuit_transaction_10x2.json

npx circom circuit.circom -o circuit.json && npx snarkjs info -c circuit.json
# npx snarkjs info -c circuit.json



# initialize ceremony
# cargo run --release --bin new circuit.json circom1.params ./
cargo run --release --bin new circuit-by-circom-2.json circom1.params ./
cargo run --release --bin new transaction_2x2.json transaction_2x2_circom1.params ./
cargo run --release --bin new transaction_3x2.json transaction_3x2_circom1.params ./
cargo run --release --bin new transaction_4x2.json transaction_4x2_circom1.params ./
cargo run --release --bin new transaction_5x2.json transaction_5x2_circom1.params ./
cargo run --release --bin new transaction_6x2.json transaction_6x2_circom1.params ./
cargo run --release --bin new transaction_7x2.json transaction_7x2_circom1.params ./
cargo run --release --bin new transaction_8x2.json transaction_8x2_circom1.params ./
# cargo run --release --bin new circuit_constraints.json circom1.params ./

cargo run --release --bin contribute circom1.params circom2.params
cargo run --release --bin contribute transaction_8x2_circom2.params transaction_8x2_circom3.params
cargo run --release --bin contribute transaction_8x2_circom3.params transaction_8x2_circom4.params
cargo run --release --bin contribute transaction_8x2_circom4.params transaction_8x2_circom5.params
cargo run --release --bin contribute transaction_8x2_circom5.params transaction_8x2_circom6.params
cargo run --release --bin contribute transaction_8x2_circom6.params transaction_8x2_circom7.params
cargo run --release --bin contribute transaction_8x2_circom7.params transaction_8x2_circom8.params
cargo run --release --bin contribute transaction_8x2_circom8.params transaction_8x2_circom9.params
cargo run --release --bin contribute transaction_8x2_circom9.params transaction_8x2_circom10.params

# cargo run --release --bin verify_contribution circuit.json circom1.params circom2.params ./
cargo run --release --bin verify_contribution transaction_1x2.json transaction_2x2_circom1.params transaction_2x2_circom2.params ./

cargo run --release --bin contribute circom2.params circom3.params
# cargo run --release --bin verify_contribution circuit.json circom2.params circom3.params ./
cargo run --release --bin verify_contribution transaction_1x2.json circom2.params circom3.params ./

cargo run --release --bin contribute circom3.params circom4.params
cargo run --release --bin verify_contribution circuit.json circom3.params circom4.params ./


cp ../powersoftau/phase1radix* .
cargo run --release --bin new circuit.json circom1.params ./
cargo run --release --bin contribute circom1.params circom2.params
cargo run --release --bin contribute circom2.params circom3.params
cargo run --release --bin contribute circom3.params circom4.params
cargo run --release --bin contribute circom4.params circom5.params
cargo run --release --bin contribute circom5.params circom6.params
cargo run --release --bin contribute circom6.params circom7.params
cargo run --release --bin contribute circom7.params circom8.params
cargo run --release --bin contribute circom8.params circom9.params
cargo run --release --bin contribute circom9.params circom10.params
cargo run --release --bin contribute circom10.params circom11.params
cargo run --release --bin contribute circom11.params circom12.params
cargo run --release --bin contribute circom12.params circom13.params
cargo run --release --bin contribute circom13.params circom14.params
cargo run --release --bin contribute circom14.params circom15.params
cargo run --release --bin contribute circom15.params circom16.params
cargo run --release --bin contribute circom16.params circom17.params
cargo run --release --bin contribute circom17.params circom18.params
cargo run --release --bin contribute circom18.params circom19.params
cargo run --release --bin contribute circom19.params circom20.params



npx snarkjs groth16 setup circuit.r1cs circom4.params circuit_0000.zkey
cargo run --release --bin copy_json transaction_0001_2x2.zkey pk.json transformed_pk.json


# create dummy keys in circom format
echo "Generating dummy key files..."
npx snarkjs setup --protocol groth
# generate resulting keys
cargo run --release --bin export_keys circom4.params vk.json pk.json
# patch dummy keys with actual keys params
cargo run --release --bin copy_json proving_key.json pk.json transformed_pk.json

# generate solidity verifier
cargo run --release --bin generate_verifier circom4.params verifier.sol

# try to generate and verify proof
npx snarkjs calculatewitness
cargo run --release --bin prove circuit-by-circom-2.json witness.json circom4.params proof.json public.json
npx snarkjs verify --vk vk.json --proof proof.json


snarkjs wc circuit.wasm input.json witness.wtns
snarkjs wej witness.wtns witness.json