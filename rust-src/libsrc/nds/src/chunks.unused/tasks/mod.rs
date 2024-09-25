
mod upload;
// mod download;

use std::sync::Arc;
use async_std::io::{Read, Seek, Cursor};

pub use upload::ChunkUploader;

pub trait AsyncChunkCursor: Read + Seek {}

impl<T> AsyncChunkCursor for Cursor<T> where T: AsRef<[u8]> + Unpin {}

#[async_trait::async_trait]
pub trait TaskTrait: 'static + Sync + Send {
    fn close_as_task(&self) -> Box<dyn TaskTrait>;
}

#[derive(Clone)]
pub struct ChunkContent(Arc<Vec<u8>>);

impl From<Vec<u8>> for ChunkContent {
    fn from(data: Vec<u8>) -> Self {
        Self(Arc::new(data))
    }
}

impl AsRef<[u8]> for ChunkContent {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl ChunkContent {
    pub fn into_cursor(&self) -> Box<dyn AsyncChunkCursor + Unpin + Send + Sync> {
        Box::new(Cursor::new(self.clone()))
    }
}

mod test {
    use std::{sync::Arc, };
    use async_std::{io::ReadExt, };

    use super::ChunkContent;

    #[test]
    fn test() {
        async_std::task::block_on(async move {
            let c_orig = ChunkContent(Arc::new(vec![1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,0,89,]));
            let mut v = vec![];

            for _ in 0..3 {
                let c = c_orig.clone();
                v.push(
                    async_std::task::spawn(async move {
                        let mut r = c.into_cursor();
                
                        let mut b = [0u8;3];
                        loop {
                            match r.read(&mut b).await {
                                Ok(size) => {
                                    if size == 0 {
                                        break;
                                    } else {
                                        println!("{:?}", &b[..size]);
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                    println!("{:?}", b);
                                    break;
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                    })
                )
            }

            futures::future::join_all(v).await;
        });
    }
}
