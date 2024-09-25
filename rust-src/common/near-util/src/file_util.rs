use std::{io::Write, path::PathBuf};

use near_base::{
    file::{FileBodyContent, FileDescContent, FileObject},
    hash_util::hash_stream,
    Area, ChunkId, ErrorCode, Hash256, NearError, NearResult, ObjectBuilder, ObjectId,
    CHUNK_MAX_LEN,
};
use sha2::Digest;
// use sha2::Digest;

use crate::ReadWithLimit;

pub struct FileBuilder<'a> {
    name: Option<&'a str>,
    owner: Option<&'a ObjectId>,
    author: Option<&'a ObjectId>,
    area: Option<Area>,
    path: &'a PathBuf,
}

impl<'a> FileBuilder<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        Self {
            name: None,
            owner: None,
            author: None,
            area: None,
            path,
        }
    }

    pub fn name(mut self, name: Option<&'a str>) -> Self {
        self.name = name;
        self
    }

    pub fn owner(mut self, owner: Option<&'a ObjectId>) -> Self {
        self.owner = owner;
        self
    }

    pub fn author(mut self, author: Option<&'a ObjectId>) -> Self {
        self.author = author;
        self
    }

    pub fn area(mut self, area: Option<Area>) -> Self {
        self.area = area;
        self
    }
}

impl FileBuilder<'_> {
    pub async fn build(self) -> NearResult<FileObject> {
        let name = if let Some(name) = self.name {
            Ok(name)
        } else {
            self.path
                .file_name()
                .map(|name| name.to_str().unwrap())
                .ok_or_else(|| {
                    NearError::new(
                        ErrorCode::NEAR_ERROR_INVALIDPARAM,
                        "invalid file, it isn't file name.",
                    )
                })
        }?;

        let owner = self.owner;
        let author = self.author;
        let area = self.area;

        let f = async_std::fs::OpenOptions::new()
            .create(false)
            .read(true)
            .open(self.path.as_path())
            .await
            .map_err(|err| NearError::from(err))?;

        struct ReadChunk {
            sha256: sha2::Sha256,
            reader: ReadWithLimit,
        }

        impl async_std::io::Read for ReadChunk {
            fn poll_read(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
                buf: &mut [u8],
            ) -> std::task::Poll<std::io::Result<usize>> {
                let r = std::pin::Pin::new(&mut self.reader).poll_read(cx, buf);

                match r {
                    std::task::Poll::Ready(Ok(n)) => {
                        let _ = self.sha256.write(&buf[..n]);
                        std::task::Poll::Ready(Ok(n))
                    }
                    std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }

        impl ReadChunk {
            fn reset(&mut self) {
                self.reader.reset();
            }

            fn into_file_hash(self) -> Hash256 {
                self.sha256.finalize().into()
            }
        }

        let mut chunks = vec![];
        let mut file_len = 0;
        let mut read_chunks = ReadChunk {
            sha256: sha2::Sha256::new(),
            reader: ReadWithLimit::new(CHUNK_MAX_LEN, Box::new(f)),
        };

        let file_hash = loop {
            let (hash, chunk_len) = hash_stream(&mut read_chunks).await?;
            chunks.push(ChunkId::from((hash, chunk_len as u32)));
            file_len += chunk_len;

            if chunk_len == CHUNK_MAX_LEN {
                read_chunks.reset();
            } else {
                break read_chunks.into_file_hash();
            }
        };

        let file = ObjectBuilder::new(
            FileDescContent::new(name.to_owned(), file_len, file_hash),
            FileBodyContent::new(chunks),
        )
        .update_desc(|desc| {
            desc.set_owner(owner.cloned());
            desc.set_author(author.cloned());
            desc.set_area(area);
        })
        .build()?;

        Ok(file)
    }
}

#[test]
fn test() {
    use near_base::hash_file;

    async_std::task::block_on(async move {
        let p =
            std::path::PathBuf::new().join("C:\\cyfs\\log\\app\\sn-miner-rust_137648_r00036.log");

        let (hash, _h) = hash_file(p.as_path()).await.unwrap();
        println!("{}", hash);

        match FileBuilder::new(&p).build().await {
            Ok(f) => {
                println!("file: {}", f)
            }
            Err(e) => {
                println!("err: {}", e);
            }
        }
    });
}
