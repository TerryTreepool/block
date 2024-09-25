
pub mod aes_key;

pub mod private_key;
pub mod public_key;
pub mod signature;

pub use aes_key::AesKey;
pub use private_key::PrivateKey;
pub use public_key::PublicKey;
pub use signature::Signature;

use async_trait::async_trait;

use crate::NearResult;

#[async_trait]
pub trait SignerTrait: Send + Sync {
    fn public_key(&self) -> &PublicKey;
    async fn sign(&self, data: &[u8]) -> NearResult<Signature>;
}

#[async_trait]
impl SignerTrait for Box<dyn SignerTrait> {
    fn public_key(&self) -> &PublicKey {
        self.as_ref().public_key()
    }

    async fn sign(&self, data: &[u8]) -> NearResult<Signature> {
        self.as_ref().sign(data).await
    }
}

#[async_trait]
pub trait VerifierTrait: Send + Sync {
    // fn public_key(&self) -> &PublicKey;
    async fn verify(&self, data: &[u8], sign: &Signature) -> NearResult<()>;
}

#[async_trait]
impl VerifierTrait for Box<dyn VerifierTrait> {
    // fn public_key(&self) -> &PublicKey {
    //     self.as_ref().public_key()
    // }

    async fn verify(&self, data: &[u8], sign: &Signature) -> NearResult<()> {
        self.as_ref().verify(data, sign).await
    }
}
