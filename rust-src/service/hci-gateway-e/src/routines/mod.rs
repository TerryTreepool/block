
pub mod schedule;
pub mod things;

use bytes::*;

#[derive(Debug)]
pub struct Request {
    pub opcode: u16,
    pub controller: u16,
    pub param: Bytes,
}

impl From<Request> for Bytes {
    fn from(val: Request) -> Self {
        let mut buf = BytesMut::with_capacity(6 + val.param.len());

        buf.put_u16_le(val.opcode as u16);
        buf.put_u16_le(val.controller.into());
        buf.put_u16_le(val.param.len() as u16);
        buf.put(val.param);

        buf.freeze()
    }
}

#[test]
fn test_request() { 
    let r = Request {
        opcode: 0x3E,
        controller: 1,
        param: Bytes::new(),
    };

    let v = Bytes::from(r);
    println!("{:?}", v.as_ref());
}

