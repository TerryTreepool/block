
use near_base::{aes_key::KeyMixHash, AesKey, Sequence};


lazy_static::lazy_static!{
    static ref CHANNEL_NO: Sequence = Sequence::random();
}

#[derive(Clone)]
pub(super) struct ProxyStub {
    channel_key: KeyMixHash,
}

impl ProxyStub {
    pub fn new() -> Self {
        let channel_no = CHANNEL_NO.generate().into_value();

        Self {
            channel_key: AesKey::generate().mix_hash(Some(channel_no as u64)),
        }
    }

    #[inline]
    pub fn channel_key(&self) -> &KeyMixHash {
        &self.channel_key
    }

    #[inline]
    pub fn into_channel_key(self) -> KeyMixHash {
        self.channel_key
    }
}
