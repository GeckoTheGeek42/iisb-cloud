use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub fn encrypt(pass: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.input_str(pass);
	hasher.result_str()
}

pub fn check(input: &str, pass: &str) -> bool {
	&encrypt(input) == pass
}