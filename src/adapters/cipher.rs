use crate::{error::ServiceError, state::SharedCipher};

pub struct CipherAdapter;

impl CipherAdapter {
    pub async fn decipher(cipher: &SharedCipher, raw: &str) -> Result<String, ServiceError> {
        cipher.decipher(raw).await
    }
}
