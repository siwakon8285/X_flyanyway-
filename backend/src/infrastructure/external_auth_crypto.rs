use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{
    application::external_auth::ExternalCredentialCrypto,
    domain::external_api::{
        AccessTokenHash, CredentialDigest, ExternalApiCredentialPepper, PlaintextAccessToken,
        PlaintextClientSecret,
    },
};

type HmacSha256 = Hmac<Sha256>;

const CREDENTIAL_DOMAIN_SEPARATOR: &[u8] = b"x-fly-client-credential-v1\0";
const DUMMY_SECRET: [u8; 32] = [0; 32];

#[derive(Clone)]
pub struct HmacExternalCredentialCrypto {
    pepper: ExternalApiCredentialPepper,
}

impl HmacExternalCredentialCrypto {
    pub fn from_pepper(pepper: ExternalApiCredentialPepper) -> Self {
        Self { pepper }
    }

    fn digest_for(&self, public_client_id: &str, secret: &[u8; 32]) -> CredentialDigest {
        let mut input = Vec::with_capacity(
            CREDENTIAL_DOMAIN_SEPARATOR.len() + public_client_id.len() + 1 + secret.len(),
        );
        input.extend_from_slice(CREDENTIAL_DOMAIN_SEPARATOR);
        input.extend_from_slice(public_client_id.as_bytes());
        input.push(0);
        input.extend_from_slice(secret);

        let mut mac = HmacSha256::new_from_slice(self.pepper.as_bytes())
            .expect("HMAC-SHA-256 accepts a 32-byte pepper");
        mac.update(&input);
        let digest: [u8; 32] = mac.finalize().into_bytes().into();
        CredentialDigest::from_bytes(digest)
    }
}

impl ExternalCredentialCrypto for HmacExternalCredentialCrypto {
    fn generate_client_secret(&self) -> PlaintextClientSecret {
        let mut bytes = [0_u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        PlaintextClientSecret::from_bytes(bytes)
    }

    fn credential_digest(
        &self,
        public_client_id: &str,
        secret: &PlaintextClientSecret,
    ) -> CredentialDigest {
        self.digest_for(public_client_id, secret.as_bytes())
    }

    fn dummy_credential_digest(&self, public_client_id: &str) -> CredentialDigest {
        self.digest_for(public_client_id, &DUMMY_SECRET)
    }

    fn digest_matches(&self, supplied: &CredentialDigest, stored: &CredentialDigest) -> bool {
        supplied.as_bytes().ct_eq(stored.as_bytes()).into()
    }

    fn generate_access_token(&self) -> PlaintextAccessToken {
        let mut bytes = [0_u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        PlaintextAccessToken::from_bytes(bytes)
    }

    fn access_token_hash(&self, token: &PlaintextAccessToken) -> AccessTokenHash {
        let hash: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        AccessTokenHash::from_bytes(hash)
    }
}
