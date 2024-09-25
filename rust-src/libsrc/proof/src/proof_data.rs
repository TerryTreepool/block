
use near_base::{PrivateKey, NearResult, PublicKey, Signature, 
                proof_of_data::{ProofOfDataObject, ProofOfDataDescContent, ProofOfDataBodyContent}, 
                DeviceObject, Serialize, Deserialize, ObjectBuilder, NearError, ErrorCode};
use protos::proof::ping::{Proof_of_ping, Proof_of_device};

#[async_trait::async_trait]
pub trait ProofOfProcessTrait {
    async fn sign(&self, private_key: PrivateKey) -> NearResult<Signature>;
    async fn verify(&self, public_key: &PublicKey, signature: &Signature) -> NearResult<()>;
}

pub struct ProofOfDataReq<T, C>
where T: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default,
      C: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default {
    proof_of_data: ProofOfDataObject<T, C>,
    proof: Signature,
}

impl<T, C> ProofOfDataReq<T, C> 
where T: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default,
      C: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default {

    pub fn split(self) -> (ProofOfDataObject<T, C>, Signature) {
        (self.proof_of_data, self.proof)
    }
}

impl<T, C> Serialize for ProofOfDataReq<T, C> 
where T: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default,
      C: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default {

    fn raw_capacity(&self) -> usize {
        self.proof_of_data.raw_capacity() + 
        self.proof.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.proof_of_data.serialize(buf)?;
        let buf = self.proof.serialize(buf)?;

        Ok(buf)
    }
}

impl<T, C> Deserialize for ProofOfDataReq<T, C>
where T: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default,
      C: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default {

    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (proof_of_device, buf) = ProofOfDataObject::<T, C>::deserialize(buf)?;
        let (proof, buf) = Signature::deserialize(buf)?;

        Ok((Self {
            proof_of_data: proof_of_device, proof,
        }, buf))
    }

}

impl<T, C> std::fmt::Display for ProofOfDataReq<T, C> 
where T: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default,
      C: Clone + std::fmt::Display + Serialize + Deserialize + std::default::Default {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data: {}, sign: {{***}}", self.proof_of_data)
    }
}

const PROOFDATASET_OF_PUBLISH: u8       = 1u8;
const PROOFDATASET_OF_PING: u8          = 2u8;

pub enum ProofDataSet {
    Publish(ProofOfDataReq<Proof_of_device, DeviceObject>),
    Ping(ProofOfDataReq<Proof_of_ping, DeviceObject>),
}

impl ProofDataSet {
    pub async fn with_publish(proof_of_device: Proof_of_device, device_object: DeviceObject, private_key: PrivateKey) -> NearResult<Self> {

        let proof = proof_of_device.sign(private_key).await?;

        let proof_of_data = 
            ObjectBuilder::new(ProofOfDataDescContent::default(), ProofOfDataBodyContent::default())
                .update_desc(| mut_desc | {
                    mut_desc.no_create_time();
                    mut_desc.mut_desc().set_proof_data(proof_of_device);
                })
                .update_body(| mut_body | {
                    mut_body.mut_body().set_data(device_object);
                })
                .build()?;

        Ok(Self::Publish(ProofOfDataReq{
            proof_of_data,
            proof,
        }))
    }

    pub async fn with_ping(
        proof_of_ping: Proof_of_ping, 
        device_object: DeviceObject, 
        private_key: PrivateKey
    ) -> NearResult<Self> {
        let proof = proof_of_ping.sign(private_key).await?;

        let proof_of_data = 
            ObjectBuilder::new(ProofOfDataDescContent::default(), ProofOfDataBodyContent::default())
                .update_desc(| mut_desc | {
                    mut_desc.no_create_time();
                    mut_desc.mut_desc().set_proof_data(proof_of_ping);
                })
                .update_body(| mut_body | {
                    mut_body.mut_body().set_data(device_object);
                })
                .build()?;

        Ok(Self::Ping(ProofOfDataReq{
            proof_of_data,
            proof,
        }))
    }
}

impl Serialize for ProofDataSet {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::Publish(data) => {
                PROOFDATASET_OF_PUBLISH.raw_capacity() + data.raw_capacity()
            }
            Self::Ping(data) => {
                PROOFDATASET_OF_PING.raw_capacity() + data.raw_capacity()
            }
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::Publish(data) => {
                let buf = PROOFDATASET_OF_PUBLISH.serialize(buf)?;
                let buf = data.serialize(buf)?;

                Ok(buf)
            }
            Self::Ping(data) => {
                let buf = PROOFDATASET_OF_PING.serialize(buf)?;
                let buf = data.serialize(buf)?;

                Ok(buf)
            }
        }
    }
}

impl Deserialize for ProofDataSet {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (f, buf) = u8::deserialize(buf)?;

        match f {
            PROOFDATASET_OF_PUBLISH => {
                let (data, buf) = ProofOfDataReq::<Proof_of_device, DeviceObject>::deserialize(buf)?;

                Ok((Self::Publish(data), buf))
            }
            PROOFDATASET_OF_PING => {
                let (data, buf) = ProofOfDataReq::<Proof_of_ping, DeviceObject>::deserialize(buf)?;

                Ok((Self::Ping(data), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("undefined id [{f}].")))
            }
        }
    }
}

impl std::fmt::Display for ProofDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Publish(proof) => {
                write!(f, "proof-publish: {}", proof)
            }
            Self::Ping(proof) => {
                write!(f, "proof-ping: {}", proof)
            }
        }
    }
}

impl std::fmt::Debug for ProofDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}
