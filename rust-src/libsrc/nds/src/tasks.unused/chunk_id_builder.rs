use near_base::{Serialize, ObjectId, Timestamp, now, NearResult, hash_data};

use crate::nds_protocol::ChunkPieceDesc;

pub struct ChunkTaskIdBuilder<'a, ID: Serialize> {
    target: Option<&'a ObjectId>,
    id: Option<&'a ID>,
    desc: Option<&'a ChunkPieceDesc>,
    now: Timestamp,
}

impl<ID: Serialize> std::default::Default for ChunkTaskIdBuilder<'_, ID> {
    fn default() -> Self {
        Self {
            target: None,
            id: None,
            desc: None,
            now: now(),
        }
    }
}

impl<'a, ID: Serialize> ChunkTaskIdBuilder<'a, ID> {
    pub fn set_target(mut self, target: &'a ObjectId) -> Self {
        self.target = Some(target);
        self
    }

    pub fn set_id(mut self, id: &'a ID) -> Self {
        self.id = Some(id);
        self
    }

    pub fn set_desc(mut self, desc: &'a ChunkPieceDesc) -> Self {
        self.desc = Some(desc);
        self
    }
}

impl<'a, ID: Serialize> ChunkTaskIdBuilder<'a, ID> {
    pub fn build(self) -> NearResult<ObjectId> {
        let len = 
            self.target.map(| target | target.raw_capacity()).unwrap_or(0) +
            self.id.map(| id | id.raw_capacity()).unwrap_or(0) +
            self.desc.map(| desc | desc.raw_capacity()).unwrap_or(0) + 
            self.now.raw_capacity();

        let mut buf = vec![0u8; len];
        let end = buf.as_mut_slice();

        let end = if let Some(target) = self.target {
            target.serialize(end)?
        } else {
            end
        };
        let end = if let Some(id) = self.id {
            id.serialize(end)?
        } else {
            end
        };
        let end = if let Some(desc) = self.desc {
            desc.serialize(end)?
        } else {
            end
        };
        let end = self.now.serialize(end)?;
        let len = len - end.len();

        Ok(hash_data(&buf[..len]).into())
    }
}
