
use log::{trace, error, warn};

use near_base::{ErrorCode, NearError, NearResult, EndpointPair, AesKey, Deserialize, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use proof::proof_data::{ProofDataSet, ProofOfProcessTrait};
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, proof::ping::Proof_of_ping_resp};

use crate::process::Process;

pub struct PingRoutine {
    process: Process,
}

impl PingRoutine {

    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(PingRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for PingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("PingRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(ProofDataSet, req, o, o, { header_meta.sequence() });

        let r: DataContent<Proof_of_ping_resp> = match r {
            DataContent::Content(proof) => self.on_routine(header_meta, proof).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl PingRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, proof: ProofDataSet) -> NearResult<Proof_of_ping_resp> {

        // get endpoint pair
        let endpoint_pair = 
            if let Some(creator) = header_meta.creator.as_ref() {
                let local = creator.creator_local.as_ref().ok_or_else(|| {
                        error!("missing local endpoint...");
                        NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing local endpoint.")
                    })?;
                let remote = creator.creator_remote.as_ref().ok_or_else(|| {
                        error!("missing remote endpoint...");
                        NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing remote endpoint.")
                    })?;

                Ok(EndpointPair::new(local.clone(), remote.clone()))
            } else {
                error!("missing creator info...");
                Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing creator."))
            }?;

        let (proof_of_ping, proof_of_device, proof) = 
            match proof {
                ProofDataSet::Ping(proof) => {
                    let (mut proof_of_data, proof) = proof.split();

                    Ok((
                        proof_of_data.mut_desc().mut_content().take_proof_data(),
                        proof_of_data.mut_body().mut_content().take_data(),
                        proof
                    ))
                },
                _ => {
                    let error_string = "error proof";
                    error!("{error_string}, sequence: {}", header_meta.sequence());
                    Err(NearError::new(ErrorCode::NEAR_ERROR_FATAL, error_string))
                }
            }?;

        // verify
        if let Some(public_key) = proof_of_device.desc().public_key() {
            proof_of_ping.verify(&public_key, &proof)
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })
        } else {
            error!("missing public key, sequence: {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_FATAL, "missing public key"))
        }?;

        if !self.process.peer_manager()
                .peer_heartbeat(
                    proof_of_device,
                    proof,
                    // aes-key
                    {
                        let (key, _) = 
                            AesKey::deserialize(proof_of_ping.key())
                                .map_err(| e | {
                                    error!("{e}, sequence: {}", header_meta.sequence());
                                    e
                                })?;

                        key
                    },
                    endpoint_pair.clone(),
                    proof_of_ping.send_time,
                    proof_of_ping.ping_sequence
                ) {
            warn!("{}:{} cache peer failed. the ping maybe is timeout, sequence: {}.", proof_of_ping.send_time(), proof_of_ping.ping_sequence(), header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, "the ping maybe is timeout"))
        } else {
            Ok(())
        }?;

        Ok(Proof_of_ping_resp {
            ping_sequence: proof_of_ping.ping_sequence(),
            local_endpoint: endpoint_pair.local().to_string(),
            remote_endpoint: endpoint_pair.remote().to_string(),
            ..Default::default()
        })
    }

}
