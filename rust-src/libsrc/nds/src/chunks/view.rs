
use std::{sync::{Arc, }, path::Path, };

use log::error;
use near_base::{ChunkId, NearResult, ErrorCode, NearError, };

use crate::{nds_protocol::{PieceEncodeDesc, PieceMessage, }, inc::{SaveToPathTrait}, };

use super::{super::inc::{ChunkReaderTrait, ChunkWriterTrait
                }, ChunkAccess, 
        };

enum ViewStateImpl {
    R(Box<dyn ChunkReaderTrait>),
    W(Box<dyn ChunkWriterTrait>, Box<dyn SaveToPathTrait>),
    // R(MemChunk),
    // W(RwLock<MemChunk>),
}

struct ChunkViewImpl {
    chunk: ChunkId,
    state: ViewStateImpl,
}

#[derive(Clone)]
pub struct ChunkView(Arc<ChunkViewImpl>);

impl ChunkView {
    pub fn with_readonly(chunk: ChunkId, r: Box<dyn ChunkReaderTrait>) -> Self {
        Self(Arc::new(ChunkViewImpl{
            chunk,
            state: ViewStateImpl::R(r),
        }))
    }

    pub fn with_write<W: ChunkWriterTrait + SaveToPathTrait + 'static>(chunk: ChunkId, w: W) -> Self {
        Self(Arc::new(ChunkViewImpl{
            chunk,
            state: ViewStateImpl::W(w.clone_as_writer(), Box::new(w)),
        }))
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

    pub fn access_mode(&self) -> ChunkAccess {
        match self.0.state {
            ViewStateImpl::R(_) => ChunkAccess::Read,
            ViewStateImpl::W(_, _) => ChunkAccess::Write,
        }
    }

    pub async fn read(&self, offset: usize, length: usize) -> NearResult<Vec<u8>> {
        debug_assert!(offset < length, "fatal");

        let chunk_len = self.chunk().len();

        if length > chunk_len {
            let error_string = format!("failed read data with except length, length = {}, chunk-len = {}", length, chunk_len);
            error!("{}", error_string);
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_string))
        } else {
            let get_size = length - offset;
            let mut buff = vec![0u8; get_size];
            match self.read_to_end(buff.as_mut(), offset).await {
                Ok(count) => {
                    if count == get_size {
                        Ok(buff)
                    } else {
                        let error_string = format!("The read data length is not enough, offset = {}, length = {}, count = {}", offset, length, count);
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, error_string))
                    }
                }
                Err(err) => {
                    Err(err)
                }
            }
        }
    }

    pub async fn read_to_end(&self, buf: &mut [u8], offset: usize) -> NearResult<usize> {
        match &self.0.state {
            ViewStateImpl::R(data) => {
                data.get(self.chunk(), offset, buf).await
            }
            ViewStateImpl::W(_, _) => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, format!("Cloud not read {} data, because it's readonly.", self.chunk())))
            }
        }
    }

    pub async fn get_piece(&self, buf: &mut [u8], desc: &PieceEncodeDesc) -> NearResult<usize> {
        // match desc {
        //     PieceEncodeDesc::Range(offset, count) => {
        //         {
        //             self.0.state
        //                 .reader
        //                 .as_ref()
        //                 .map(| reader | reader.clone_as_reader() )
        //                 .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloudn't found the [{}] chunk's reader", self.chunk())))
        //         }?
        //         .get(self.chunk(), *offset as usize, *count as usize)
        //         .await
        //         .map(| r | {
        //             buf[..].copy_from_slice(r);
        //             Ok(r.len())
        //         })?
        //     }
        // }
        unimplemented!()
    }

}

#[async_trait::async_trait]
impl ChunkReaderTrait for ChunkView {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, _chunk: &ChunkId) -> bool {
        true
    }

    async fn get(&self, chunk: &ChunkId, offset: usize, content: &mut [u8]) -> NearResult<usize> {
        match &self.0.state {
            ViewStateImpl::R(reader) => {
                reader.get(chunk, offset, content).await
            }
            ViewStateImpl::W(_, _) => {
                unreachable!("The view can't read flag.");
            }
        }
    }
}

#[async_trait::async_trait]
impl ChunkWriterTrait for ChunkView {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<usize> {
        match &self.0.state {
            ViewStateImpl::W(writer, _) => {
                writer.write(chunk, offset, content).await
            }
            ViewStateImpl::R(_) => {
                unreachable!("The view can't write flag.");
            }
        }
    }

}

#[async_trait::async_trait]
impl SaveToPathTrait for ChunkView {
    async fn save_to_path(&self, path: &Path) -> NearResult<()> {
        match &self.0.state {
            ViewStateImpl::W(_, saver) => {
                saver.save_to_path(path).await
            }
            ViewStateImpl::R(_) => {
                unreachable!("The view can't write flag.");
            }
        }
    }
}

// impl async_std::io::Write for ChunkView {
//     fn poll_write(self: std::pin::Pin<&mut Self>,
//                   cx: &mut std::task::Context<'_>,
//                   buf: &[u8],
//     ) -> std::task::Poll<Result<usize>> {
//         let pined = self.get_mut();
//         let written = pined.cache.write(pined.offset, buf);
//         pined.offset += written;
//         Poll::Ready(Ok(written))
//     }

// }

// #[async_trait::async_trait]
// impl AsyncWriteWithSeek for ChunkView {
//     async fn finished(&self, _: Box<dyn ChunkWriterTrait>) {
//         match &self.0.state {
//             ViewStateImpl::W(writer) => {

//             }
//         }
//     }

//     async fn err(&self, e: NearError) {
//         unimplemented!()
//     }

// }

