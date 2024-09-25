
use log::trace;
use near_base::{*, sequence::SequenceString};
use super::MajorCommand;

lazy_static::lazy_static! {
    static ref COMMAND_CAPACITY: usize = std::mem::size_of::<u8>();
    static ref SEQUENCE_CAPACITY: usize = SequenceString::raw_bytes();
    static ref TIMESTAMP_CAPACITY: usize = std::mem::size_of::<Timestamp>();
    static ref LENGTH_CAPACITY: usize = std::mem::size_of::<u16>();
    static ref INDEX_CAPACITY: usize = std::mem::size_of::<u8>();
    static ref COUNT_CAPACITY: usize = std::mem::size_of::<u8>();
    static ref R_CAPACITY: usize = std::mem::size_of::<u8>();
    static ref PACKAGEHEADER_FIX_CAPACITY: usize =  *COMMAND_CAPACITY + 
                                                    *SEQUENCE_CAPACITY + 
                                                    *TIMESTAMP_CAPACITY +
                                                    *LENGTH_CAPACITY + 
                                                    *INDEX_CAPACITY + 
                                                    *COUNT_CAPACITY;
}

/// length = PackageHeader::USize + PackageHeaderExt::USize + PackageBody::USize + SignData::USize
#[derive(Clone)]
pub struct PackageHeader {
    major_command: MajorCommand,
    sequence: SequenceString,
    timestamp: Timestamp,
    length: u16,
    index: u8,
    count: u8,
}

impl std::fmt::Display for PackageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "major_command: {}, sequence: {}, timestamp: {}, length: {}, index: {}, count: {}", 
            self.major_command,
            self.sequence,
            self.timestamp,
            self.length,
            self.index,
            self.count)
    }
}

impl std::default::Default for PackageHeader {
    fn default() -> Self {
        Self {
            major_command: MajorCommand::None,
            sequence: Default::default(),
            timestamp: now(),
            length: 0,
            index: 0,
            count: 0,
        }
    }
}

impl PackageHeader {
    #[inline]
    pub(in crate) fn set_major_command(mut self, major_command: MajorCommand) -> Self {
        self.major_command = major_command;
        self
    }
    #[inline]
    pub(in crate) fn set_sequence(mut self, seq: SequenceString) -> Self {
        self.sequence = seq;
        self
    }
    #[inline]
    pub(in crate) fn set_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }
    #[inline]
    pub(in crate) fn set_length(mut self, length: u16) -> Self {
        self.length = length;
        self
    }
    #[inline]
    pub(in crate) fn set_index(mut self, index: u8) -> Self {
        self.index = index;
        self
    }
    #[inline]
    pub(in crate) fn set_count(mut self, count: u8) -> Self {
        self.count = count;
        self
    }

    #[inline]
    pub(in crate) fn split(self) -> (MajorCommand, SequenceString) {
        (self.major_command, self.sequence)
    }
}

impl PackageHeader {

    #[inline]
    pub fn major_command(&self) -> MajorCommand {
        self.major_command
    }
    #[inline]
    pub fn sequence(&self) -> &SequenceString {
        &self.sequence
    }
    #[inline]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    #[inline]
    pub fn length(&self) -> u16 {
        self.length
    }
    #[inline]
    pub fn index(&self) -> u8 {
        self.index
    }
    #[inline]
    pub fn count(&self) -> u8 {
        self.count
    }

}

impl RawFixedBytes for PackageHeader {
    fn raw_bytes() -> usize {
        *PACKAGEHEADER_FIX_CAPACITY
    }
}

impl Serialize for PackageHeader {
    fn raw_capacity(&self) -> usize {
        self.major_command.raw_capacity() +
        self.sequence.raw_capacity() +
        self.timestamp.raw_capacity() +
        self.length.raw_capacity() + 
        self.index.raw_capacity() +
        self.count.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.major_command.serialize(buf)?;
        let buf = self.sequence.serialize(buf)?;
        let buf = self.timestamp.serialize(buf)?;
        let buf = self.length.serialize(buf)?;
        let buf = self.index.serialize(buf)?;
        let buf = self.count.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for PackageHeader {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (major_command, buf) = u8::deserialize(buf)?;
        let (sequence, buf) = SequenceString::deserialize(buf)?;
        let (timestamp, buf) = Timestamp::deserialize(buf)?;
        let (length, buf) = u16::deserialize(buf)?;
        let (index, buf) = u8::deserialize(buf)?;
        let (count, buf) = u8::deserialize(buf)?;

        let major_command: MajorCommand = u8::try_into(major_command)?;

        Ok((Self{major_command,
                sequence,
                timestamp,
                length, 
                index,
                count,
            }, buf))
    }

}

impl From<(&PackageHeader, u8 /* index */)> for PackageHeader {
    fn from(cx: (&PackageHeader, u8 /* index */)) -> Self {
        let (head, index) = cx;

        Self {
            major_command: head.major_command.clone(),
            sequence: head.sequence.clone(),
            timestamp: now(),
            length: 0,
            index: index,
            count: head.count,
        }
    }
}

#[derive(Clone, Default)]
pub struct PackageSource {
    /// 创建者
    pub(crate) creator: Option<ObjectId>,
    /// 请求者
    pub(crate) requestor: ObjectId,
    /// 请求者本地addr
    pub(crate) creator_local: Option<Endpoint>,
    /// 请求者远端addr
    pub(crate) creator_remote: Option<Endpoint>,
}

impl PackageSource {

    #[inline]
    pub fn creator(&self) -> Option<&ObjectId> {
        self.creator.as_ref()
    }

    #[inline]
    pub fn requestor(&self) -> &ObjectId {
        &self.requestor
    }

    #[inline]
    pub fn creator_local(&self) -> Option<&Endpoint> {
        self.creator_local.as_ref()
    }

    #[inline]
    pub fn creator_remote(&self) -> Option<&Endpoint> {
        self.creator_remote.as_ref()
    }
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "creator={:?}, addr:[local={:?}, remote:{:?}], requestor={:?}", 
            self.creator, 
            self.creator_local(),
            self.creator_remote(),
            self.requestor
        )
    }
}

#[derive(Clone)]
pub struct PackageHeaderExt {
    pub(crate) from: PackageSource,
    pub(crate) to: ObjectId,
    pub(crate) topic: Option<String>, /* minjor command */
}

impl std::fmt::Display for PackageHeaderExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "from=[{}], to={}, topic={:?}", self.from, self.to, self.topic)
    }
}

impl std::default::Default for PackageHeaderExt {
    fn default() -> Self {
        Self {
            from: Default::default(),
            to: Default::default(),
            topic: None,
        }
    }
}

impl PackageHeaderExt {

    #[inline]
    pub(crate) fn set_creator(mut self, creator: Option<ObjectId>) -> Self {
        self.from.creator = creator;
        self
    }

    #[inline]
    pub(crate) fn set_requestor(mut self, requestor: ObjectId) -> Self {
        self.from.requestor = requestor;
        self
    }

    #[inline]
    pub(crate) fn set_endpoint(mut self, local: Option<Endpoint>, remote: Option<Endpoint>) -> Self {
        self.from.creator_local = local;
        self.from.creator_remote = remote;
        self
    }

    #[inline]
    pub(crate) fn set_to(mut self, to: ObjectId) -> Self {
        self.to = to;
        self
    }

    #[inline]
    pub(crate) fn set_topic(mut self, topic: Option<String>) -> Self {
        self.topic = topic;
        self
    }
}

impl PackageHeaderExt {

    #[inline]
    pub fn source(&self) -> &PackageSource {
        &self.from
    }

    #[inline]
    pub fn creator(&self) -> Option<&ObjectId> {
        self.from.creator.as_ref()
    }

    #[inline]
    pub fn requestor(&self) -> &ObjectId {
        &self.from.requestor
    }

    #[inline]
    pub fn endpoints(&self) -> (Option<&Endpoint>, Option<&Endpoint>) {
        (self.from.creator_local(), self.from.creator_remote())
    }

    #[inline]
    pub fn to(&self) -> &ObjectId {
        &self.to
    }

    #[inline]
    pub fn topic(&self) -> Option<&String> {
        self.topic.as_ref()
    }

    #[inline]
    pub fn split(self) -> (PackageSource, ObjectId, Option<String>) {
        (self.from, self.to, self.topic)
    }
}

impl Serialize for PackageHeaderExt {
    fn raw_capacity(&self) -> usize {
        self.from.creator.raw_capacity() + 
        self.from.creator_local.raw_capacity() + 
        self.from.creator_remote.raw_capacity() +
        self.from.requestor.raw_capacity() + 
        self.to.raw_capacity() + 
        self.topic.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.from.creator.serialize(buf)?;
        let buf = self.from.creator_local.serialize(buf)?;
        let buf = self.from.creator_remote.serialize(buf)?;
        let buf = self.from.requestor.serialize(buf)?;
        let buf = self.to.serialize(buf)?;
        let buf = self.topic.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for PackageHeaderExt {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        trace!("PackageHeaderExt::deserialize len: {}", buf.len());

        let (creator, buf) = Option::<ObjectId>::deserialize(buf)?;
        let (creator_local, buf) = Option::<Endpoint>::deserialize(buf)?;
        let (creator_remote, buf) = Option::<Endpoint>::deserialize(buf)?;
        let (requestor, buf) = ObjectId::deserialize(buf)?;
        let (to, buf) = ObjectId::deserialize(buf)?;
        let (topic, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            from: PackageSource { creator, creator_local, creator_remote, requestor }, 
            to, 
            topic,
        }, buf))
    }

}
