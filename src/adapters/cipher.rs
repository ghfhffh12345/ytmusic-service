use crate::{error::ServiceError, state::SharedCipher};

pub struct CipherAdapter;

impl CipherAdapter {
    pub async fn decipher(cipher: &SharedCipher, raw: &str) -> Result<String, ServiceError> {
        cipher.decipher(raw).await.map_err(|error| match error {
            ServiceError::CipherOperation(source) => ServiceError::Cipher(source),
            other => other,
        })
    }
}
