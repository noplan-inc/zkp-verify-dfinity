use bellman::groth16::{prepare_verifying_key, Proof, verify_proof, VerifyingKey};
use bls12_381::{Bls12, Scalar};
use base64::{decode};
use std::convert::TryInto;

fn pop(bytes: Vec<u8>) -> [u8; 32] {
    bytes.try_into().expect("failed to convert")
}

#[ic_cdk_macros::query]
fn verify(vk_base64: String, proof_base64: String, image_base64: String) -> bool {
    let vk_bytes = decode(vk_base64).unwrap();
    let proof_bytes = decode(proof_base64).unwrap();
    let image_vec = decode(image_base64).unwrap();

    let image_bytes = pop(image_vec);

    let vk_bls12 = VerifyingKey::<Bls12>::read(&vk_bytes[..]).expect("failed to read VerifyingKey");

    let proof = Proof::read(&proof_bytes[..]).expect("failed to read Proof");

    let image = Scalar::from_bytes(&image_bytes).unwrap();

    let pvk = prepare_verifying_key(&vk_bls12);
    verify_proof(&pvk, &proof, &[image]).is_ok()
}