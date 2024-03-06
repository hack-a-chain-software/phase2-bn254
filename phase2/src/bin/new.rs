extern crate rand;
extern crate phase2;
extern crate exitcode;

use std::fs::File;
use phase2::parameters::MPCParameters;
use phase2::circom_circuit::circuit_from_json_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("pre args");
    if args.len() != 4 {
        println!("Usage: \n<in_circuit.json> <out_params.params> <path/to/phase1radix>");
        std::process::exit(exitcode::USAGE);
    }

    println!("let circuit");
    let circuit_filename = &args[1];
    let params_filename = &args[2];
    let radix_directory = &args[3];

    let should_filter_points_at_infinity = false;

    println!("Creating initial radix_directory for {}...", radix_directory);
    println!("Creating initial params_filename for {}...", params_filename);
    println!("Creating initial circuit_filename for {}...", circuit_filename);

    let c = circuit_from_json_file(&circuit_filename);

    println!("circuit is here now");

    let params = MPCParameters::new(c, should_filter_points_at_infinity, radix_directory).unwrap();

    println!("Writing initial parameters to {}.", params_filename);
    let mut f = File::create(params_filename).unwrap();
    params.write(&mut f).expect("unable to write params");
}
