
use near_base::*;

use protos::proof::ping::Proof_of_device;

use crate::proof_data::ProofOfProcessTrait;

#[async_trait::async_trait]
impl ProofOfProcessTrait for Proof_of_device {
    async fn sign(&self, private_key: PrivateKey) -> NearResult<Signature> {

        let signer_buffer = [
            &self.send_time.to_be_bytes(),
            &self.ping_sequence.to_be_bytes(),
            self.nonce.as_bytes()
        ].concat();

        private_key.sign(&signer_buffer)
    }

    async fn verify(&self, public_key: &PublicKey, signature: &Signature) -> NearResult<()> {

        let signer_buffer = [
            &self.send_time.to_be_bytes(),
            &self.ping_sequence.to_be_bytes(),
            self.nonce.as_bytes()
        ].concat();

        public_key.verify(&signer_buffer, signature)
    }

}
