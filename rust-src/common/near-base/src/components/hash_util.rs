
use std::{io::Write, path::Path};
use std::io::ErrorKind;

use async_std::io::ReadExt;
use sha2::Digest;

use crate::{Hash256, NearResult, NearError};

pub fn hash_data(data: &[u8]) -> Hash256 {
    let mut sha256 = sha2::Sha256::new();
    let _ = sha256.write(data);
    sha256.finalize().as_slice().into()
}

pub async fn hash_stream(reader: &mut (impl ReadExt + Unpin), ) -> NearResult<(Hash256, u64)> {
    let mut sha256 = sha2::Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut len = 0;

    loop {
        match reader.read(&mut buf).await {
            Ok(size) => {
                if size == 0 {
                    break;
                }
                let _ = sha256.write(&buf[0..size]);
                len = len + size;
            }
            Err(e) => {
                if let ErrorKind::Interrupted = e.kind() {
                    continue; // Interrupted
                }
                return Err(NearError::from(e));
            }
        }
    }

    Ok((sha256.finalize().as_slice().into(), len as u64))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn hash_file(path: &Path) -> NearResult<(Hash256, u64)> {

    let mut file = async_std::fs::File::open(path).await?;

    let mut buf = vec![0u8; 64*1024];
    let mut sha256 = sha2::Sha256::new();

    let mut file_len = 0;
    loop {
        match file.read(&mut buf).await {
            Ok(size) => {
                if size == 0 {
                    break;
                }
                let _ = sha256.write(&buf[0..size]);
                file_len = file_len + size;
            }
            Err(e) => {
                if let ErrorKind::Interrupted = e.kind() {
                    continue; // Interrupted
                }
                return Err(NearError::from(e));
            }
        }
    }

    Ok((sha256.finalize().as_slice().into(), file_len as u64))
}

#[test]
fn test_hash_file() {
    async_std::task::block_on(async move {
        let p = std::path::PathBuf::new().join("C:\\cyfs\\log\\app\\sn-miner-rust_137648_r00037.log");

        if let Ok((h, l)) = hash_file(p.as_path()).await {
            println!("{}", h);
            println!("{}", l);

        }
    });
}
