use nanopulse::keypair;

pub fn run() {
    let seed = keypair::get_seed();
    let (pk, secret) = keypair::new(seed);

    println!("Public key: {}", hex::encode(pk.to_bytes()));
    println!("Secret: {}", hex::encode(secret.to_bytes()));
}
