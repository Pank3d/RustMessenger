use {
	crate::states::Account,
	aes_siv::{aead::Aead, AeadInPlace, Aes128SivAead, KeyInit, Nonce},
	ed25519_dalek::VerifyingKey,
	std::path::PathBuf,
};

pub(crate) fn small_public_key(pk: &VerifyingKey) -> String {
	let mut ret = bs58::encode(pk.to_bytes()).into_string();
	ret.replace_range(6..ret.len() - 6, "..");
	ret
}

pub(crate) fn public_key(pk: &VerifyingKey) -> String {
	bs58::encode(pk.to_bytes()).into_string()
}

pub(crate) fn data_path(basedir: PathBuf, hash: String) -> PathBuf {
	basedir.join("data").join(&hash[0..4]).join(&hash[4..8])
}

pub(crate) fn encrypt(account: &Account, opposite: &VerifyingKey, mut buf: Vec<u8>) -> Vec<u8> {
	let nonce = rand::random::<[u8; 16]>();
	Aes128SivAead::new_from_slice(&x25519_dalek::x25519(
		account.secret.to_scalar_bytes(),
		opposite.to_montgomery().0,
	))
	.unwrap()
	.encrypt_in_place(Nonce::from_slice(&nonce), b"", &mut buf)
	.unwrap();
	let mut ret = Vec::<u8>::new();
	ret.extend(b"aes-256-siv");
	ret.extend(nonce);
	ret.extend(buf);
	ret
}

pub(crate) fn decrypt(account: &Account, opposite: &VerifyingKey, data: &Vec<u8>) -> String {
	if data.starts_with(b"aes-256-siv") {
		let offset = b"aes-256-siv".len();
		let Some(nonce) = data.get(offset..offset + 16) else {
			return String::from("Failed to decrypt this message")
		};

		let Ok(buf) = Aes128SivAead::new_from_slice(&x25519_dalek::x25519(
			account.secret.to_scalar_bytes(),
			opposite.to_montgomery().0,
		))
		.unwrap()
		.decrypt(Nonce::from_slice(nonce), &data[offset + 16..]) else {
			return String::from("Failed to decrypt this message due decryption")
		};

		return String::from_utf8(buf).unwrap_or(String::from("Failed to decrypt this message"))
	}

	String::from("Cannot get message format")
}
