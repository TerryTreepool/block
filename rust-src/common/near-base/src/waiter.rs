
// use async_std::future;
use futures::future::{AbortHandle, AbortRegistration, Abortable, self};

pub struct StateWaiter {
    wakers: Vec<AbortHandle>,
}

impl StateWaiter {
    pub fn new() -> Self {
        Self { wakers: vec![] }
    }

    pub fn transfer(&mut self) -> Self {
        let mut waiter = Self::new();
        self.transfer_into(&mut waiter);
        waiter
    }

    pub fn transfer_into(&mut self, waiter: &mut Self) {
        waiter.wakers.append(&mut self.wakers);
    }

    pub fn new_waiter(&mut self) -> AbortRegistration {
        let (waker, waiter) = AbortHandle::new_pair();
        self.wakers.push(waker);
        waiter
    }

    pub async fn wait<T, S: FnOnce() -> T>(waiter: AbortRegistration, state: S) -> T {
        let _ = Abortable::new(future::pending::<()>(), waiter).await;
        state()
    }

    pub fn wake(self) {
        for waker in self.wakers {
            waker.abort();
        }
    }

    pub fn len(&self) -> usize {
        self.wakers.len()
    }
}

#[cfg(test)]
mod test {
    use std::sync::{RwLock, Arc};
    use super::StateWaiter;

    #[test]
    fn test() {
        struct TestStructImpl {
            share_vec: Vec<u8>,
            waiter: StateWaiter,
        }

        let arc_test = Arc::new(RwLock::new(TestStructImpl{
            share_vec: vec![],
            waiter: StateWaiter::new(),
        }));

        {
            let mut t = vec![1,2,3,4];
            let c = vec![100,200,300];
            let _p = 1;

            t.extend_from_slice(&c);

            // t.resize_with(10, || { p *= 2;p });

            println!("{:?}", t);
        }

        // let waiter = {
        //     arc_test.write().unwrap().waiter.new_waiter()
        // };
        let count = 10;
        let _ = 
        {
            let arc_clone = arc_test.clone();

            async_std::task::spawn(async move {
                for i in 0..count {
                    let to_waiter = {
                        let mut_clone = &mut *arc_clone.write().unwrap();
                        mut_clone.waiter.new_waiter()
                    };

                    let r = StateWaiter::wait(to_waiter, ||{ 
                        let vec = &mut arc_clone.write().unwrap().share_vec;
                        vec.pop()
                    }).await.unwrap();

                    println!("{}: {}", i, r);
                }
            })
        };

        let _ = 
        {
            let arc_clone = arc_test.clone();

            async_std::task::block_on( async move {
                for i in 0..count {
                    std::thread::sleep(std::time::Duration::from_secs(2));

                    let to_wake = {
                        let mut_clone = &mut *arc_clone.write().unwrap();
                        let vec = &mut mut_clone.share_vec;
                        vec.push(i as u8);

                        mut_clone.waiter.transfer()
                    };
                    to_wake.wake();
                }
            })
        };

        std::thread::sleep(std::time::Duration::from_secs(10));

    }
}