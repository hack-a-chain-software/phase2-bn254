extern crate hex;
use powersoftau::{
    batched_accumulator::BatchedAccumulator,
    keypair::keypair,
    parameters::{CeremonyParams, CheckForCorrectness, UseCompression},
    utils::calculate_hash,
};

use bellman_ce::pairing::bn256::Bn256;
use memmap::MmapOptions;
use std::fs::OpenOptions;
use memmap::MmapMut;
use std::io::BufWriter;
use std::fs::File;

use std::io::Write;
extern crate hex_literal;

const INPUT_IS_COMPRESSED: UseCompression = UseCompression::No;
const COMPRESS_THE_OUTPUT: UseCompression = UseCompression::Yes;
const CHECK_INPUT_CORRECTNESS: CheckForCorrectness = CheckForCorrectness::No;

#[allow(clippy::modulo_one)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 7 {
        println!("Usage: \n<challenge_file> <response_file> <circuit_power> <batch_size> <beacon_hash> <num_iterations_exp>");
        std::process::exit(exitcode::USAGE);
    }
    let challenge_filename = &args[1];
    let response_filename = &args[2];
    let circuit_power = args[3].parse().expect("could not parse circuit power");
    let batch_size = args[4].parse().expect("could not parse batch size");
    let beacon_hash = &args[5];
    let num_iterations_exp = &args[6].parse::<usize>().unwrap();

    if *num_iterations_exp < 10 || *num_iterations_exp > 63 {
        println!("in_num_iterations_exp should be in [10, 63] range");
        std::process::exit(exitcode::DATAERR);
    }

    let parameters = CeremonyParams::<Bn256>::new(circuit_power, batch_size);

    println!(
        "Will contribute a random beacon to accumulator for 2^{} powers of tau",
        parameters.size,
    );
    println!(
        "In total will generate up to {} powers",
        parameters.powers_g1_length,
    );

    // Create an RNG based on the outcome of the random beacon
    let mut rng = {
        use byteorder::{BigEndian, ReadBytesExt};
        use crypto::digest::Digest;
        use crypto::sha2::Sha256;
        use rand::chacha::ChaChaRng;
        use rand::SeedableRng;

        let mut cur_hash = hex::decode(beacon_hash).unwrap();

        // Performs 2^n hash iterations over it
        let n: usize = *num_iterations_exp;

        for i in 0..(1u64 << n) {
            // Print 1024 of the interstitial states
            // so that verification can be
            // parallelized

            if i % (1u64 << (n - 10)) == 0 {
                print!("{}: ", i);
                for b in cur_hash.iter() {
                    print!("{:02x}", b);
                }
                println!();
            }

            let mut h = Sha256::new();
            h.input(&cur_hash);
            h.result(&mut cur_hash);
        }

        print!("Final result of beacon: ");
        for b in cur_hash.iter() {
            print!("{:02x}", b);
        }
        println!();

        let mut digest = &cur_hash[..];

        let mut seed = [0u32; 8];
        for s in &mut seed {
            *s = digest
                .read_u32::<BigEndian>()
                .expect("digest is large enough for this to work");
        }

        ChaChaRng::from_seed(&seed)
    };

    println!("Done creating a beacon RNG");

    // Try to load challenge file from disk.
    let reader = OpenOptions::new()
        .read(true)
        .open(challenge_filename)
        .expect("unable open challenge file in this directory");

    {
        let metadata = reader
            .metadata()
            .expect("unable to get filesystem metadata for challenge file");
        let expected_challenge_length = match INPUT_IS_COMPRESSED {
            UseCompression::Yes => parameters.contribution_size,
            UseCompression::No => parameters.accumulator_size,
        };

        if metadata.len() != (expected_challenge_length as u64) {
            panic!(
                "The size of challenge file should be {}, but it's {}, so something isn't right.",
                expected_challenge_length,
                metadata.len()
            );
        }
    }

    let readable_map = unsafe {
        MmapOptions::new()
            .map(&reader)
            .expect("unable to create a memory map for input")
    };

    // Create response file in this directory
    let mut response_writer = BufWriter::new(
        File::create(response_filename).expect("unable to create response file")
    );

    let required_output_length = match COMPRESS_THE_OUTPUT {
        UseCompression::Yes => parameters.contribution_size,
        UseCompression::No => parameters.accumulator_size + parameters.public_key_size,
    };

    let mut writable_map = vec![0; required_output_length];

    println!("Calculating previous contribution hash...");

    let current_accumulator_hash = calculate_hash(&readable_map);

    {
        println!("Contributing on top of the hash:");
        for line in current_accumulator_hash.as_slice().chunks(16) {
            print!("\t");
            for section in line.chunks(4) {
                for b in section {
                    print!("{:02x}", b);
                }
                print!(" ");
            }
            println!();
        }

        (&mut writable_map[0..])
            .write_all(current_accumulator_hash.as_slice())
            .expect("unable to write a challenge hash to mmap");

        writable_map
            .flush()
            .expect("unable to write hash to response file");
    }

    // Construct our keypair using the RNG we created above
    let (pubkey, privkey) = keypair(&mut rng, current_accumulator_hash.as_ref());

    // Perform the transformation
    println!("Computing and writing your contribution, this could take a while...");

    let mmap_mut_ptr: *mut MmapMut = &mut writable_map as *mut Vec<u8> as *mut MmapMut;

    // this computes a transformation and writes it
    unsafe {
        BatchedAccumulator::transform(
            &readable_map,
            &mut *mmap_mut_ptr,
            INPUT_IS_COMPRESSED,
            COMPRESS_THE_OUTPUT,
            CHECK_INPUT_CORRECTNESS,
            &privkey,
            &parameters,
        )
        .expect("must transform with the key");
    }
    
    println!("Finishing writing your contribution to response file...");

    // Write the public key
    unsafe {
        pubkey
            .write(&mut *mmap_mut_ptr, COMPRESS_THE_OUTPUT, &parameters)
            .expect("unable to write public key");
    }

    // Write the processed data to the response file
    response_writer
        .write_all(&writable_map)
        .expect("unable to write to response file");
    response_writer
        .flush()
        .expect("unable to flush response file");

    print!(
        "Done!\n\n\
              Your contribution has been written to response file\n\n\
              The BLAKE2b hash of response file is:\n"
    );

    println!("Thank you for your participation, much appreciated! :)");
}
