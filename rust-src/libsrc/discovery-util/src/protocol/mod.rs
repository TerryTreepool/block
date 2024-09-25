
use std::any::Any;

use near_base::{Serialize, Deserialize, NearResult};

pub mod v0;

pub trait BodyTrait: Send + Sync + Serialize { }

type PackageBodyTrait = Box<dyn Any + Send + Sync>;

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    #[default]
    None = 0,
    Search = 1,
    SearchResp = 2,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Search => write!(f, "search"),
            Self::SearchResp => write!(f, "search resp")
        }
    }
}

impl TryFrom<u8> for Command {
    type Error = near_base::NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1u8 => Ok(Command::Search),
            2u8 => Ok(Command::SearchResp),
            _ => Err(near_base::NearError::new(near_base::ErrorCode::NEAR_ERROR_UNDEFINED, format!("undefined [{value}] command")))
        }        
    }
}

#[derive(Default)]
pub struct Head {
    pub ver: u8,
    pub cmd: Command,
    pub uid: u32,
}

impl near_base::Serialize for Head {
    fn raw_capacity(&self) -> usize {
        self.ver.raw_capacity() + 
        (self.cmd as u8).raw_capacity() + 
        self.uid.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = self.ver.serialize(buf)?;
        let buf = (self.cmd as u8).serialize(buf)?;
        let buf = self.uid.serialize(buf)?;

        Ok(buf)
    }
}

impl near_base::Deserialize for Head {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (ver, buf) = u8::deserialize(buf)?;
        let (cmd, buf) = u8::deserialize(buf)?;
        let (uid, buf) = u32::deserialize(buf)?;

        Ok((Self {
            ver,
            cmd: cmd.try_into()?,
            uid,
        }, buf))
    }
}

pub struct ParsePackage {
    head: Head,
    body: PackageBodyTrait,
}

unsafe impl Send for ParsePackage {}
unsafe impl Sync for ParsePackage {}

impl ParsePackage {

    pub fn parse(text: &[u8]) -> near_base::NearResult<Self> {
        let (head, text) = Head::deserialize(&text)?;

        let data = 
            match head.cmd {
                Command::Search => {
                    v0::Search::deserialize(text)
                        .map(| (data, _) | {
                            Box::new(data) as PackageBodyTrait
                        })?
                }
                Command::SearchResp => {
                    v0::SearchResp::deserialize(text)
                        .map(| (data, _) | {
                            Box::new(data) as PackageBodyTrait
                        })?
                }
                _ => unimplemented!()
            };

        Ok(Self {
            head,
            body: data
        })

    }

    pub fn take_head(&mut self) -> Head {
        std::mem::replace(&mut self.head, Default::default())
    }

    pub fn take_body<T>(&mut self) -> T where T: 'static + BodyTrait + std::default::Default {
        let body = self.body.downcast_mut::<T>().unwrap();

        std::mem::replace(body, T::default())
    }

}

impl<B> From<(Head, B)> for ParsePackage
where B: 'static + BodyTrait {
    fn from(value: (Head, B)) -> Self {
        let (head, body) = value;    

        Self {
            head, 
            body: Box::new(body),
        }    
    }
}

pub struct BuildPackage {
    pub head: Head,
    pub body: Box<dyn BodyTrait>,
}

impl BuildPackage {
    pub fn build(self) -> NearResult<Vec<u8>> {
        let cap = self.head.raw_capacity() + self.body.raw_capacity();
        let mut buf = vec![0u8; cap];

        {
            let _dst = &mut buf;
            let _dst = self.head.serialize(_dst)?;
            let _dst = self.body.serialize(_dst)?;
        }

        Ok(buf)
    }
}
