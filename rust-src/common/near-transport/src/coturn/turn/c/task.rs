
use std::{sync::Arc, time::Duration};

use log::error;
use near_base::{aes_key::KeyMixHash, sequence::SequenceString, Endpoint, ErrorCode, NearError, NearResult, ObjectId, RawFixedBytes, Serialize};

use crate::{
        coturn::turn::p::{ProxyDatagramTrait, ProxyInterface}, 
        network::UdpPackageEventTrait, 
        package::{package_decode, PackageDataSet}, 
        tunnel::PostMessageTrait, 
        PayloadMaxLen, 
        Stack
    };

use super::key::TurnMixHash;

struct TaskImpl {
    stack: Stack,
    proxy_interface: Option<ProxyInterface>,
    mix_hash_stubs: TurnMixHash,
}

#[derive(Clone)]
pub struct Task(Arc<TaskImpl>);

impl Task {
    pub fn open(stack: Stack) -> NearResult<Self> {
        let this = 
            Self(Arc::new(TaskImpl{
                stack,
                proxy_interface: None,
                mix_hash_stubs: TurnMixHash::new(),
            }));

        let proxy_interface = ProxyInterface::open(None, Box::new(this.clone()) as Box<dyn ProxyDatagramTrait>)?;

        unsafe {
            &mut *(Arc::as_ptr(&this.0) as *mut TaskImpl)
        }
        .proxy_interface = Some(proxy_interface);

        let arc_self = this.clone();
        async_std::task::spawn(async move {
            arc_self.process().await;
        });

        Ok(this)
    }

    pub(crate) fn mix_hash_stubs(&self) -> TurnMixHash {
        self.0.mix_hash_stubs.clone()
    }

    pub(in self) async fn process(&self) {
        let heartbeat = {
                
            let mut buf = [0u8; PayloadMaxLen];
            let option_mixhash = Option::<KeyMixHash>::None;
            let r = 
                option_mixhash.serialize(&mut buf)
                    .map(| remain | PayloadMaxLen - remain.len() )
                    .unwrap();
            buf[..r].to_vec()
        };

        loop {
            let proxy_interface = self.0.proxy_interface.as_ref().unwrap();
            let proxy_address = self.mix_hash_stubs().proxy_addresses();

            {
                let mut futs = vec![];

                for address in proxy_address.iter() {
                    futs.push(proxy_interface.send_to(heartbeat.as_ref(), address));
                }
    
                let _ = futures::future::join_all(futs).await;
            }

            let _ = 
                async_std::future::timeout(
                    Duration::from_secs(30),
                    async_std::future::pending::<()>(),
                )
                .await;
        }
    }
}

#[async_trait::async_trait]
impl ProxyDatagramTrait for Task {

    async fn on_proxied_datagram(
        &self, 
        mix_hash: near_base::aes_key::KeyMixHash, 
        datagram: &[u8], 
        from: Endpoint
    ) {
        
        let mix_hash_length = mix_hash.raw_capacity();
        let datagram = datagram[mix_hash_length..].to_vec();
        let task = self.clone();
        let stack = self.0.stack.clone();

        async_std::task::spawn(async move {
            let data_context = 
                match package_decode::decode_package(datagram.as_slice()).await {
                    Ok(data_context) => data_context,
                    Err(err) => {
                        error!("failed decode package mix-hash: {mix_hash}, with err: {err}");
                        return;
                    }
                };

            let _ = stack.on_udp_package(task.0.proxy_interface.as_ref().unwrap().udp(), data_context, from).await;

        });

    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(ObjectId, SequenceString, PackageDataSet)> for Task {

    type R = ();

    async fn post_message(
        &self, 
        context: (ObjectId, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {

        let (sequence, package, stub) = {
            let (target, sequence, mut package) = context;

            let stub = 
                self.0.mix_hash_stubs.get(&target)
                    .ok_or_else(|| {
                        let error_string = format!("missing proxy target stub.");
                        error!("{error_string}, sequence: {sequence}");
                        NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                    })?;

            let mut package_array = vec![];
            let package_data = package.take_dataset();
            // check package space
            for (_, data) in package_data.iter() {
                let data_len = data.as_ref().len();
                let package_len = 1 + data_len + KeyMixHash::raw_bytes();
                if package_len > PayloadMaxLen {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, "package space not enough"));
                }

                let mut payload = [0u8; PayloadMaxLen];
                let mut_payload = 1u8.serialize(&mut payload)?;
                let mut_payload = stub.mix_hash.serialize(mut_payload)?;
                mut_payload[..data_len].copy_from_slice(data.as_ref());

                package_array.push(payload[..package_len].to_vec());
            }
            (sequence, package_array, stub)
        };

        let proxy_interface = 
            self.0.proxy_interface.as_ref()
                .ok_or_else(|| {
                    let error_string = format!("missing proxy interface, proxy don't startup.");
                    error!("{error_string}, sequence: {sequence}");
                    NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                })?;

        let mut futs = vec![];

        for package_data in package.iter() {
            futs.push(proxy_interface.send_to(package_data.as_slice(), &stub.proxy_address));
        }

        let _ = futures::future::join_all(futs).await;

        Ok(())
    }

            // proxy_interface.send_to(datagram, sockaddr)
        // self.get_session(Some(self.as_manager().config().call_timeout))
        //     .await?
        //     .post_message(context)
        //     .await
}

