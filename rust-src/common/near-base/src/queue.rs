
use std::{sync::{Mutex, Arc}, time::Duration, collections::VecDeque, 
    };

use async_std::future;

use crate::StateWaiter;


type QueueVecPtr<DATA> = Arc<VecDeque<DATA>>;

enum QueueState<DATA> {
    Waiting(WaitingState<DATA>),
    Working(WorkingState<DATA>),
}

struct WaitingState<DATA> {
    // packages: QueueVecPtr,
    queues: QueueVecPtr<DATA>,
    waiter: StateWaiter,
}

struct WorkingState<DATA> {
    queues: QueueVecPtr<DATA>,
}

pub struct Queue<DATA> {
    state: Mutex<QueueState<DATA>>,
}

impl<DATA> std::default::Default for Queue<DATA> {
    fn default() -> Self {
        Self {
            state: Mutex::new(QueueState::Waiting(WaitingState{
                queues: Arc::new(VecDeque::new()),
                waiter: StateWaiter::new(),
            }))
        }
    }
}

pub enum PushMethod {
    PushHead,
    PushTail,
}

impl<DATA> Queue<DATA> {

    pub fn push(&self, data: DATA, m: Option<PushMethod>) {
        let to_wake = {
            let state  = &mut *self.state.lock().unwrap();
            match state {
                QueueState::Waiting(waiting_state) => {
                    let queues = Arc::get_mut(&mut waiting_state.queues).unwrap();
                    let to_wake = waiting_state.waiter.transfer();
                    match m.unwrap_or(PushMethod::PushTail) {
                        PushMethod::PushHead => queues.push_front(data),
                        PushMethod::PushTail => queues.push_back(data)
                    }
                    *state = QueueState::Working(WorkingState{
                        queues: waiting_state.queues.clone(),
                    });

                    Some(to_wake)
                }
                QueueState::Working(working_state) => {
                    let queues = Arc::get_mut(&mut working_state.queues).unwrap();
                    match m.unwrap_or(PushMethod::PushTail) {
                        PushMethod::PushHead => queues.push_front(data),
                        PushMethod::PushTail => queues.push_back(data)
                    }
                    None
                }
            }
        };

        if let Some(to_wake) = to_wake {
            to_wake.wake();
        }
    }

    pub fn take(&self) -> Option<DATA> {
        let package = {
            let state = &mut *self.state.lock().unwrap();
            match state {
                QueueState::Waiting(_waiting_state) => {
                    None
                }
                QueueState::Working(working_state) => {
                    let packages = Arc::get_mut(&mut working_state.queues).unwrap();
                    let package = packages.pop_front();

                    if packages.len() == 0 {
                        *state = QueueState::Waiting( WaitingState { queues: working_state.queues.clone(), waiter: StateWaiter::new() })
                    }
                    package
                }
            }
        };

        package
    }

    pub async fn wait(&self, timeout: Duration) -> bool {
        let waiter = {
            match &mut *self.state.lock().unwrap() {
                QueueState::Waiting(waiting_state) => {
                    Some(waiting_state.waiter.new_waiter())
                }
                QueueState::Working(_working_state) => {
                    None
                }
            }
        };

        if let Some(waiter) = waiter {
            future::timeout(timeout, StateWaiter::wait(waiter, ||{ true }))
                .await
                .unwrap_or(false)
        } else {
            true
        }
    }

    pub async fn wait_and_take(&self, timeout: Duration) -> Option<DATA> {
        if self.wait(timeout).await {
            self.take()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct QueueGuard<DATA: Clone>(Arc<Queue<DATA>>);

impl<DATA: Clone> std::default::Default for QueueGuard<DATA> {
    fn default() -> Self {
        Self(Arc::new(Queue::default()))
    }
}

impl<DATA: Clone> std::ops::Deref for QueueGuard<DATA> {
    type Target = Queue<DATA>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()        
    }
}

mod test {
    #[test]
    fn test_queue() {
        async_std::task::block_on(async move {
            // use near_base::{ExtentionObject, ObjectId, builder_codec::FileDecoder, now, ObjectGuard, SequenceValue, };
            // use near_core::get_data_path;
        
            // use crate::package::*;
        
            #[derive(Clone, Debug)]
            struct Data {
                index: u32,
            }

            use super::QueueGuard;
        
            let queue = QueueGuard::<Data>::default();

            let mut va = vec![];

            for _ in 0..2 {
                let queue_clone = queue.clone();
                va.push(async_std::task::spawn(async move {
                    let mut count = 0;
                    loop {
                        if queue_clone.wait(std::time::Duration::from_secs(1)).await  {
                            println!("queue have data");

                            if let Some(data) = queue_clone.take() {
                                count = count + 1;
                                println!("i can take {} data = {}", count, data.index);

                                if count == 30 {
                                    break;
                                }
                            } else {
                                println!("i can't take data");
                            }
                        } else {
                            // println!("queue have not data");
                        }
                    }
                }));
            }

            va.push(async_std::task::spawn(async move {
                for idx in 0..30 {
                    // let builder = 
                    //     PackageBuilder::build_head(SequenceValue::default())
                    //         .build_topic(ObjectId::default(), None, Some("/test".to_string()))
                    //         .build_body(Ack {
                    //             result: 0,
                    //             send_time: now(),
                    //         })
                    //         .build(None)
                    //         .await
                    //         .unwrap();

                    // let package = DynamicPackage::from((head, head_ext, body, None));

                    queue.push(Data { index: idx }, None);
                    // queue.push(ObjectGuard::Extention(target.clone()), 
                    //            Some("/test".to_string()), 
                    //            Ack {
                    //                 result: 0,
                    //                 send_time: now(),
                    //             });

                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
            }));

            let _ = futures::future::join_all(va).await;
        });
    }
}