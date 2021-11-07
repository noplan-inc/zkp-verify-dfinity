use std::fs::File;
use std::io::{Write};
// For randomness (during paramgen and proof generation)
use rand::thread_rng;

// For benchmarking
use std::time::{Duration, Instant};

// Bring in some tools for using finite fiels
use ff::Field;

// We're going to use the BLS12-381 pairing-friendly elliptic curve.
use bls12_381::{Bls12, Scalar};

// We're going to use the Groth16 proving system.
use bellman::groth16::{create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof, Proof};

mod common;
use common::*;

fn main() {
    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let mut rng = thread_rng();

    // Generate the MiMC round constants
    let constants = (0..MIMC_ROUNDS)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();

    println!("Creating parameters...");

    // Create parameters for our circuit
    let params = {
        let c = MiMCDemo {
            xl: None,
            xr: None,
            constants: &constants,
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };


    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    println!("Creating proofs...");

    // Let's benchmark stuff!
    // testなので1回で良い
    const SAMPLES: u32 = 1;
    let mut total_proving = Duration::new(0, 0);
    let mut total_verifying = Duration::new(0, 0);

    // Just a place to put the proof data, so we can
    // benchmark deserialization.
    let mut proof_vec = vec![];

    for _ in 0..SAMPLES {
        // Generate a random preimage and compute the image
        let xl = Scalar::random(&mut rng);
        let xr = Scalar::random(&mut rng);
        let image = mimc(xl, xr, &constants);

        proof_vec.truncate(0);

        let start = Instant::now();
        {
            // Create an instance of our circuit (with the
            // witness)
            let c = MiMCDemo {
                xl: Some(xl),
                xr: Some(xr),
                constants: &constants,
            };

            // Create a groth16 proof with our parameters.
            let proof = create_random_proof(c, &params, &mut rng).unwrap();

            proof.write(&mut proof_vec).unwrap();
        }

        total_proving += start.elapsed();

        let start = Instant::now();
        let proof = Proof::read(&proof_vec[..]).unwrap();


        // fileにverifyに必要な情報たちを書き出し
        let mut file = File::create("mimc.proof").expect("failed to create");
        file.write_all(&proof_vec).expect("failed to write");


        let vk = File::create("verification.zk").expect("failed to create");
        params.vk.write(vk).expect("failed to write vk");

        let mut img = File::create("image.txt").expect("failed to create image.txt");
        img.write_all(&image.to_bytes()).expect("failed to write");

        assert!(verify_proof(&pvk, &proof, &[image]).is_ok());
        total_verifying += start.elapsed();
    }
    let proving_avg = total_proving / SAMPLES;
    let proving_avg =
        proving_avg.subsec_nanos() as f64 / 1_000_000_000f64 + (proving_avg.as_secs() as f64);

    let verifying_avg = total_verifying / SAMPLES;
    let verifying_avg =
        verifying_avg.subsec_nanos() as f64 / 1_000_000_000f64 + (verifying_avg.as_secs() as f64);

    println!("Average proving time: {:?} seconds", proving_avg);
    println!("Average verifying time: {:?} seconds", verifying_avg);
}

