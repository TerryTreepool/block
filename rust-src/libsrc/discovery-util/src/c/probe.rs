
use std::{sync::{Arc, Mutex}, net::{UdpSocket, SocketAddr}, time::Duration};

use near_base::{now, NearResult, NearError, ErrorCode, StateWaiter};

use crate::{GROUP_HOST, GROUP_PORT, protocol::{v0::{Search, SearchResp}, Head, BuildPackage, ParsePackage}};

use super::{Configure, ProbeResult};

struct ProbeState {
    result: Option<NearResult<ProbeResult>>,
    waiter: StateWaiter,
}

struct ProbeImpl {
    config: Configure,
    state: Mutex<ProbeState>,
    process_fut: Mutex<Option<async_std::task::JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct Probe(Arc<ProbeImpl>);

impl Probe {
    pub fn new(config: Option<Configure>) -> Self {
        Self(Arc::new(ProbeImpl{
            config: config.unwrap_or_default(),
            state: Mutex::new(ProbeState {
                result: None,
                waiter: StateWaiter::new(),
            }),
            process_fut: Mutex::new(None),
        }))
    }

    #[inline]
    pub fn config(&self) -> &Configure {
        &self.0.config
    }

    pub async fn wait(&self) -> NearResult<ProbeResult> {
        let waiter = {
            let state = &mut *self.0.state.lock().unwrap();

            match &state.result {
                Some(result) => {
                    return 
                        match result {
                            Ok(r) => Ok(r.clone()),
                            Err(e) => Err(e.clone()),
                        };
                }
                None => {
                    state.waiter.new_waiter()
                }
            }
        };

        let this = self.clone();
        StateWaiter::wait(waiter, || {
            match this.0.state.lock().unwrap().result.as_ref().unwrap() {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(e.clone()),
            }
        })
        .await
    }
}

impl Probe {
    pub async fn run(&self) -> NearResult<()> {

        let this = self.clone();

        {
            let process_fut_w = &mut *self.0.process_fut.lock().unwrap();

            match process_fut_w {
                Some(_) => Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, "already startup.")),
                None => {
                    *process_fut_w = 
                        Some(async_std::task::spawn(async move {
                            let r = 
                                this.process().await
                                    .map(| v | ProbeResult{desc_list: v.into_iter().map(| i | i.desc).collect()} );

                            let waker = {
                                let state_w = &mut *this.0.state.lock().unwrap();

                                state_w.result = Some(r);
                                state_w.waiter.transfer()
                            };

                            waker.wake();
                        }));
                    Ok(())
                }
            }

        }

    }

    async fn process(&self) -> NearResult<Vec<SearchResp>> {
        let socket = UdpSocket::bind("0.0.0.0:8888").unwrap();

        struct DataSource {
            pub remote: SocketAddr,
            pub data: SearchResp,
        }
        let mut data_list: Vec<DataSource> = vec![];

        socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();
        let current_now = now();
        let recv_timeout = self.0.config.recv_timeout.as_micros() as u64;

        let probe_data = 
            BuildPackage{
                head: Head {
                        ver: 1u8,
                        cmd: crate::protocol::Command::Search,
                        uid: current_now as u32,
                    },
                body: Box::new(Search{})}
            .build()?;

        let host = format!("{}:{}", GROUP_HOST, GROUP_PORT);

        loop {
            if now() - current_now > recv_timeout {
                break(Ok(data_list.into_iter().map(| source | source.data).collect()))
                // break(Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, "timeout")));
            }

            socket.send_to(&probe_data, host.as_str()).unwrap();

            let mut buf = [0u8; 2048];
            if let Ok((s, remote)) = socket.recv_from(&mut buf) {
                match ParsePackage::parse(&buf[..s]) {
                    Ok(mut package) => {
                        let head = package.take_head();

                        if head.uid != current_now as u32 {
                            continue;
                        }
                        if head.cmd != crate::protocol::Command::SearchResp {
                            break(Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("Unprocessable [{}] commands", head.cmd))));
                        }

                        if data_list.iter().find(| &source | {
                            if remote == source.remote {
                                return true;
                            } else {
                                return false;
                            }
                        }).is_none() {
                            let body = package.take_body::<SearchResp>();
                            println!("host:{remote}, data-id:{}", body.desc.object_id());
                            data_list.push(DataSource{
                                remote,
                                data: body
                            });
                        }
                        continue;
                    }
                    Err(e) => {
                        break(Err(e))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    
    #[test]
    fn test_discovery() {
        let _ = 
        async_std::task::block_on(async move {
            let probe = super::Probe::new(None);

            probe.run().await.unwrap();

            match probe.wait().await {
                Ok(r) => {
                    for (i, desc) in r.desc_list.iter().enumerate() {
                        println!("[{i}]: {desc}");
                        println!("++++++++++++++++++end++++++++++++++++++");
                    }
                }
                Err(e) => println!("error: {}", e),
            }

        });
    }
}
