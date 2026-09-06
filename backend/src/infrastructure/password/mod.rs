use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::RngCore;

#[derive(Clone)]
pub struct Argon2PasswordService {
    params: Params,
}

impl Default for Argon2PasswordService {
    fn default() -> Self {
        Self {
            params: Params::new(19_456, 2, 1, None).expect("valid Argon2 parameters"),
        }
    }
}

impl Argon2PasswordService {
    fn engine(&self) -> Argon2<'_> {
        Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone())
    }

    pub fn hash(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let mut salt = [0_u8; 16];
        rand::rng().fill_bytes(&mut salt);
        let salt = SaltString::encode_b64(&salt)?;
        self.engine()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    }

    pub fn verify(&self, password: &str, encoded: &str) -> bool {
        PasswordHash::new(encoded).ok().is_some_and(|hash| {
            self.engine()
                .verify_password(password.as_bytes(), &hash)
                .is_ok()
        })
    }

    pub fn needs_rehash(&self, encoded: &str) -> bool {
        let Ok(hash) = PasswordHash::new(encoded) else {
            return true;
        };
        hash.algorithm.as_str() != "argon2id"
            || hash.version != Some(Version::V0x13.into())
            || hash.params.get_decimal("m") != Some(self.params.m_cost())
            || hash.params.get_decimal("t") != Some(self.params.t_cost())
            || hash.params.get_decimal("p") != Some(self.params.p_cost())
    }
}
