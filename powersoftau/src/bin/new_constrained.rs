use powersoftau::batched_accumulator::BatchedAccumulator;
use powersoftau::parameters::{CeremonyParams, UseCompression};
use powersoftau::utils::{blank_hash};

use bellman_ce::pairing::bn256::Bn256;
use std::fs::OpenOptions;
use std::io::Write;
use memmap::MmapMut;

const COMPRESS_NEW_CHALLENGE: UseCompression = UseCompression::No;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        println!("Usage: \n<challenge_file> <ceremony_size> <batch_size>");
        std::process::exit(exitcode::USAGE);
    }
    let challenge_filename = &args[1];
    let circuit_power = args[2].parse().expect("could not parse circuit power");
    let batch_size = args[3].parse().expect("could not parse batch size");

    let parameters = CeremonyParams::<Bn256>::new(circuit_power, batch_size);

    println!(
        "Will generate an empty accumulator for 2^{} powers of tau",
        parameters.size
    );
    println!(
        "In total will generate up to {} powers",
        parameters.powers_g1_length
    );

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(challenge_filename)
        .expect("unable to create challenge file");

    let expected_challenge_length = match COMPRESS_NEW_CHALLENGE {
        UseCompression::Yes => parameters.contribution_size - parameters.public_key_size,
        UseCompression::No => parameters.accumulator_size,
    };

    file.set_len(expected_challenge_length as u64)
        .expect("unable to allocate large enough file");

    let mut writable_vec = vec![0; expected_challenge_length];

    // Write a blank BLAKE2b hash:
    let hash = blank_hash();
    writable_vec[0..hash.len()].copy_from_slice(hash.as_slice());

    println!("Blank hash for an empty challenge:");
    for line in hash.as_slice().chunks(16) {
        print!("\t");
        for section in line.chunks(4) {
            for b in section {
                print!("{:02x}", b);
            }
            print!(" ");
        }
        println!();
    }

    let mmap_mut_ptr: *mut MmapMut = &mut writable_vec as *mut Vec<u8> as *mut MmapMut;

    unsafe {
        BatchedAccumulator::generate_initial(&mut *mmap_mut_ptr, COMPRESS_NEW_CHALLENGE, &parameters)
            .expect("generation of initial accumulator is successful");
    }

    file.write_all(&writable_vec)
        .expect("unable to write to challenge file");

    println!("Wrote a fresh accumulator to challenge file");
}
