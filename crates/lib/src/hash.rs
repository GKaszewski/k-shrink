use sha2::{Digest, Sha256};

pub fn image_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        let data = b"some image bytes";
        assert_eq!(image_hash(data), image_hash(data));
    }

    #[test]
    fn different_input_different_hash() {
        assert_ne!(image_hash(b"abc"), image_hash(b"xyz"));
    }

    #[test]
    fn empty_input_no_panic() {
        let h = image_hash(b"");
        assert_eq!(h.len(), 32);
    }
}
