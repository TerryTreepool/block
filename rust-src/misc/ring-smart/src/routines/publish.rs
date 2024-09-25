
use log::{trace, error, info};

use near_base::{ErrorCode, NearError, NearResult, builder_codec_macro::Empty, ObjectBuilder, device::{DeviceDescContent, DeviceBodyContent}, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use proof::proof_data::{ProofDataSet, ProofOfProcessTrait};
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::{process::Process, cahces::DeviceObjectItem};

pub struct PublishRoutine {
    process: Process,
}

impl PublishRoutine {

    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(PublishRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for PublishRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("PublishRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(ProofDataSet, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(proof) => self.on_routine(header_meta, proof).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl PublishRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, proof: ProofDataSet) -> NearResult<Empty> {

        let (mut proof_of_data, proof) = match proof {
            ProofDataSet::Publish(proof) => Ok(proof.split()),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_FATAL, "error proof"))
        }?;

        let proof_of_device = proof_of_data.mut_desc().mut_content().take_proof_data();
        let device = DeviceObjectItem::from(proof_of_data.mut_body().mut_content().take_data());

        let mut device_transaction = 
            self.process.device_storage()
                .begin()
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        match device_transaction.load_with_prefix(device.object_id().to_string().as_str()).await {
            Ok(local_device) => {

                if local_device.desc().create_timestamp() != device.desc().create_timestamp() {
                    let error_string = "Core element[create timestamp] conflicts";
                    error!("{error_string}, sequence: {}", header_meta.sequence());
                    Err(NearError::new(ErrorCode::NEAR_ERROR_CONFLICT, error_string))
                } else {
                    Ok(())
                }?;

                if local_device.desc().owner() != device.desc().owner() {
                    let error_string = "Core element[owner] conflicts";
                    error!("{error_string}, sequence: {}", header_meta.sequence());
                    Err(NearError::new(ErrorCode::NEAR_ERROR_CONFLICT, error_string))
                } else {
                    Ok(())
                }?;

                let public_key = 
                    local_device.desc().public_key().ok_or_else(|| {
                        error!("missing public key, sequence: {}", header_meta.sequence());
                        NearError::new(ErrorCode::NEAR_ERROR_FATAL, "missing public key")
                    })?;

                // update
                proof_of_device.verify(public_key, &proof)
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?;

                let device_new = 
                    ObjectBuilder::new(DeviceDescContent::default(), DeviceBodyContent::default())
                        .update_desc(| mut_desc | {
                            mut_desc.set_create_timestamp(device.desc().create_timestamp());
                            mut_desc.set_expired_time(device.desc().expired_time());
                            mut_desc.set_owner(device.desc().owner().cloned());
                            mut_desc.set_author(device.desc().author().cloned());
                            mut_desc.set_area(device.desc().area().cloned());
                            mut_desc.set_public_key(public_key.clone());
                        })
                        .update_body(| mut_body | {
                            mut_body.mut_body().set_endpoints(device.body().content().endpoints().clone());
                            mut_body.mut_body().set_turn_node_list(device.body().content().turn_node_list().clone());
                            mut_body.mut_body().set_name(device.body().content().name());
                            mut_body.mut_body().set_stun_node_list(device.body().content().stun_node_list().clone());
                            mut_body.mut_body().set_userdata(device.body().content().userdata().clone());
                        })
                        .build()
                        .map_err(| e | {
                            error!("failed update with err: {e}, sequence: {}", header_meta.sequence());
                            e
                        })?;

                device_transaction.update(&device_new.into())
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?;

                Ok(())
            }
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => {
                        let public_key = 
                            device.desc().public_key().ok_or_else(|| {
                                error!("missing public key, sequence: {}", header_meta.sequence());
                                NearError::new(ErrorCode::NEAR_ERROR_FATAL, "missing public key")
                            })?;

                        proof_of_device.verify(public_key, &proof).await
                            .map_err(| e | {
                                error!("{e}, sequence: {}", header_meta.sequence());
                                e
                            })?;

                        device_transaction.create_new(&device).await
                            .map_err(| e | {
                                error!("{e}, sequence: {}", header_meta.sequence());
                                e
                            })?;

                        Ok(())
                    },
                    _ => {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        Err(e)
                    }
                }
            }
        }?;

        device_transaction.commit().await.map_err(| e | { error!("{e}, sequence: {}", header_meta.sequence()); e })?;

        info!("Successfully publish device: {}", device.object_id());

        Ok(Empty)
    }

}
