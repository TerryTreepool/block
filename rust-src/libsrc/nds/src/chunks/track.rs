
use std::path::PathBuf;

use log::{debug, error};
use near_base::{NearResult, Area, ObjectId};
use near_util::FileBuilder;


pub struct TrackBuilder<'a> {
    owner: Option<&'a ObjectId>,
    author: Option<&'a ObjectId>,
    area: Option<Area>,
    file: PathBuf,
}

impl std::default::Default for TrackBuilder<'_> {
    fn default() -> Self {
        Self {
            owner: None,
            author: None,
            area: None,
            file: Default::default(),
        }
    }
}

impl<'a> TrackBuilder<'a> {
    pub fn file(mut self, file: PathBuf) -> Self {
        self.file = file;
        self
    }

    pub fn owner(mut self, owner: &'a ObjectId) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn author(mut self, author: &'a ObjectId) -> Self {
        self.author = Some(author);
        self
    }

    pub fn area(mut self, area: Area) -> Self {
        self.area = Some(area);
        self
    }
}

impl TrackBuilder<'_> {
    pub async fn build(self) -> NearResult<()> {
        let file = match FileBuilder::default()
                                .owner(self.owner)
                                .area(self.area)
                                .author(self.author)
                                .path(self.file)
                                .build()
                                .await {
            Ok(file) => {
                debug!("create file object: {}", file.object_id());
                Ok(file)
            }
            Err(e) => {
                error!("failed create file object with err {}", e);
                Err(e)
            }
        }?;

        for (index, chunk) in file.body().content().chunk_list().iter().enumerate() {
            debug!("{}-{}", index, chunk);
        }

        unimplemented!()
    }
}

#[test]
fn test_track_builder() {
    async_std::task::block_on(async move {
        let _ = TrackBuilder::default()
        .file(PathBuf::new().join("C:\\cyfs\\log\\app\\sn-miner-rust_137648_r00036.log"))
        .build()
        .await;
    })
}
