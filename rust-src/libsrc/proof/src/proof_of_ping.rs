
use near_base::*;

use protos::proof::ping::Proof_of_ping;

use crate::proof_data::ProofOfProcessTrait;

#[async_trait::async_trait]
impl ProofOfProcessTrait for Proof_of_ping {
    async fn sign(&self, private_key: PrivateKey) -> NearResult<Signature> {

        let signer_buffer = if self.contract_id.len() > 0 {
            [
                &self.send_time.to_be_bytes(),
                &self.ping_sequence.to_be_bytes(),
                self.key(),
                self.contract_id().as_bytes(),
                self.nonce().as_bytes(),
            ].concat()
        } else {
            [
                &self.send_time.to_be_bytes(),
                &self.ping_sequence.to_be_bytes(),
                self.key(),
                self.nonce().as_bytes(),
            ].concat()
        };

        private_key.sign(&signer_buffer)
    }

    async fn verify(&self, public_key: &PublicKey, signature: &Signature) -> NearResult<()> {

        let signer_buffer = if self.contract_id.len() > 0 {
            [
                &self.send_time.to_be_bytes(),
                &self.ping_sequence.to_be_bytes(),
                self.key(),
                self.contract_id().as_bytes(),
                self.nonce().as_bytes(),
            ].concat()
        } else {
            [
                &self.send_time.to_be_bytes(),
                &self.ping_sequence.to_be_bytes(),
                self.key(),
                self.nonce().as_bytes(),
            ].concat()
        };

        public_key.verify(&signer_buffer, signature)
    }

}
