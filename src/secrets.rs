use crate::error::AppError;
use std::collections::BTreeMap;
use std::sync::RwLock;

pub trait SecretStore: Send + Sync {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError>;
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError>;
    fn remove(&self, reference: &str) -> Result<(), AppError>;
}

#[derive(Debug, Default)]
pub struct MemorySecretStore {
    values: RwLock<BTreeMap<String, Vec<u8>>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        Ok(self
            .values
            .read()
            .expect("secret lock poisoned")
            .get(reference)
            .cloned())
    }

    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        self.values
            .write()
            .expect("secret lock poisoned")
            .insert(reference.to_owned(), secret.to_vec());
        Ok(())
    }

    fn remove(&self, reference: &str) -> Result<(), AppError> {
        self.values
            .write()
            .expect("secret lock poisoned")
            .remove(reference);
        Ok(())
    }
}
