use std::{path::PathBuf, io::ErrorKind};

use async_std::io::ReadExt;
use log::{error};
use near_base::{ChunkId, NearResult, NearError, ErrorCode};

use super::super::store::MemChunk;

pub struct ChunkFromCache<'a> {
    chunk: &'a ChunkId,
    root_path: Option<&'a PathBuf>,
}

impl<'a> From<&'a ChunkId> for ChunkFromCache<'a> {
    fn from(chunk: &'a ChunkId) -> Self {
        Self {
            chunk, 
            root_path: None,
        }
    }


}

impl<'a> ChunkFromCache<'a> {
    pub fn path(mut self, path: &'a PathBuf) -> Self {
        self.root_path = Some(path);
        self
    }
}

impl ChunkFromCache<'_> {
    
    pub async fn get_chunk(&self) -> NearResult<MemChunk> {
        let len = self.chunk.len();
        let path = if let Some(path) = self.root_path {
            path.join(self.chunk.to_string())
        } else {
            PathBuf::new().join(self.chunk.to_string())
        };
    
        let data = match async_std::fs::OpenOptions::new()
                                    .create_new(false)
                                    .read(true)
                                    .open(path.as_path())
                                    .await {
            Ok(mut file) => {
                let mut data = vec![0u8; len];
                match file.read_to_end(&mut data)
                          .await {
                    Ok(size) => {
                        if size > len {
                            let error_string = format!("failed read chunk data with content size too long, size = {} chunk-len = {}", size, len);
                            error!("{}", error_string);
                            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_string))
                        } else if size < len {
                            let error_string = format!("failed read chunk data with content size not enough, size = {} chunk-len = {}", size, len);
                            error!("{}", error_string);
                            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_string))
                        } else {
                            Ok(data)
                        }
                    }
                    Err(err) => {
                        let error_string = format!("failed read chunk data with err = {}", err);
                        error!("{}", error_string);
                        Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
                    }
                }
            }
            Err(err) => {
                let error_string = format!("failed open chunk data with err = {}", err);
                error!("{}", error_string);
                match err.kind() {
                    ErrorKind::NotFound => {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string))
                    }
                    _ => {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
                    }
                }
            }
        }?;
    
        Ok(MemChunk::with_data(self.chunk.clone(), data))
    
    }
    
}
