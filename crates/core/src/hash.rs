pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;

    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
