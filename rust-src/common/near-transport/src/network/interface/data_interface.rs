
use std::io::ErrorKind;

use async_std::{net::TcpStream as AsyncTcpStream, io::ReadExt};
use log::error;
use near_base::*;

use crate::package::{PackageHeader, PackageHeaderExt};

use crate::network::{MTU, DataContext};

pub enum DataInterface {
    Stream(AsyncTcpStream),
    Datagram{
        inner: [u8; MTU],
        inner_used: usize,
        inner_size: usize,
    },
}

impl DataInterface {
    pub fn with_stream(stream: &AsyncTcpStream) -> Self {
        Self::Stream(stream.clone())
    }

    pub fn with_datagram(inner: [u8; MTU], inner_size: usize) -> Self {
        Self::Datagram { inner: inner, inner_used: 0, inner_size: inner_size }
    }
}

impl DataInterface {
    async fn read_exact(&mut self,
                        buf: &mut [u8]) -> NearResult<()> {
        match self {
            DataInterface::Stream(stream) => {
                let mut stream = stream.clone();
                stream.read_exact(buf)
                    .await
                    .map_err(| err | {
                        match err.kind() {
                            ErrorKind::Interrupted | ErrorKind::WouldBlock | ErrorKind::AlreadyExists | ErrorKind::TimedOut => 
                                NearError::new(ErrorCode::NEAR_ERROR_RETRY, ""),
                            _ => 
                                NearError::new(ErrorCode::NEAR_ERROR_TUNNEL_CLOSED, format!("failed recv with errno: {}", err)),
                        }
                    })?;
            },
            DataInterface::Datagram{inner, inner_used, inner_size} => {
                let buf_len = buf.len();

                if buf_len + *inner_used > *inner_size {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("out of limit buffer {}/{}:{}", buf_len, *inner_used, *inner_size)));
                }

                buf.copy_from_slice(&inner[*inner_used..*inner_used+buf_len]);
                *inner_used += buf_len;
            }
        }

        Ok(())
    }
}

impl DataInterface {
    async fn recv_package_header<'a>(&mut self, buf: &'a mut [u8]) -> NearResult<(PackageHeader, &'a mut [u8])> {
        let packet_header_len = PackageHeader::raw_bytes();

        if buf.len() < packet_header_len {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "buffer not enough"));
        }

        let header_buf = &mut buf[..packet_header_len];
        self.read_exact(header_buf)
            .await?;

        let (header, _) = PackageHeader::deserialize(header_buf)?;

        Ok((header, &mut buf[packet_header_len..]))
    }

    pub async fn recv_package(&mut self) -> NearResult<DataContext> {
        let mut recv_buf = [0u8; MTU];
    
        // recv and parse package head
        let (packet_head, remain_buf) = {
            self.recv_package_header(&mut recv_buf).await?
        };
    
        let body_length = packet_head.length() as usize - PackageHeader::raw_bytes();
        if remain_buf.len() < body_length {
            let error_message = format!("body buffer not enough, expet:{}, got:{}", remain_buf.len(), body_length);
            error!("{}", error_message);
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_message));
        }
    
        // recv and parse package body
        let data_context = {
            let body_buf = &mut remain_buf[0..body_length];
            self.read_exact(body_buf)
                .await
                .map_err(|err| {
                    let error_message = format!("failed recv packet body with errno: {}", err);
                    error!("{}", error_message);
                    NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_message)
                })?;

            // parse head-ext
            let (head_ext, remain_buf) = PackageHeaderExt::deserialize(body_buf)?;

            DataContext {
                head: packet_head,
                head_ext,
                body_data: remain_buf.to_owned(),
            }
        };
    
        Ok(data_context)
    }
}

// async fn recv_process(&self, client: &AsyncTcpStream) -> NearResult<(DynamicPackage, EndpointPair)> {
//     let mut recv_buf = [0u8; MTU];

//     let (package, _) = self.recv_package(client, &mut recv_buf).await?;

//     Ok((package, endpoint_pair))
// }

// async fn recv_package_header<'a>(&self, 
//                                  client: &AsyncTcpStream, 
//                                  builder: &mut BuilderCounter,
//                                  buf: &'a mut [u8]) -> NearResult<(PackageHeader, &'a mut [u8])> {
//     let mut client = client.clone();
//     let packet_header_len = PackageHeader::capacity();

//     if buf.len() < packet_header_len {
//         return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "buffer not enough"));
//     }

//     let header_buf = &mut buf[..packet_header_len];
//     client.read_exact(header_buf)
//           .await
//           .map_err(|err| {
//             NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
//                            format!("failed recv() with errno {}", err))
//           })?;

//     let (header, _) = PackageHeader::deserialize(header_buf, builder)?;

//     Ok((header, &mut buf[packet_header_len..]))
// }

// async fn recv_package<'a>(&self, 
//                           client: &AsyncTcpStream, 
//                           buf: &'a mut [u8]) -> NearResult<(DynamicPackage, &'a mut [u8])> {
//     let mut client = client.clone();
//     let mut builder = BuilderCounter::new();

//     // recv and parse package head
//     let (packet_head, remain_buf) = {
//         self.recv_package_header(&client, &mut builder, buf).await?
//     };

//     let body_length = packet_head.length();
//     if remain_buf.len() < body_length {
//         return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "body buffer not enough"));
//     }

//     // recv and parse package body
//     let (package, remain_buf) = {
//         let body_buf = &mut remain_buf[0..body_length];
//         client.read_exact(body_buf)
//                 .await
//                 .map_err(|err| {
//                 NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
//                                 format!("failed recv pakcet body with errno: {}", err))
//                 })?;

//         // start deserialize package
//         let (package, _) = DynamicPackage::deserialize(packet_head, &mut builder, body_buf)?;

//         (package, &mut remain_buf[body_length..])
//     };

//     Ok((package, remain_buf))
// }


#[cfg(test)]
mod test{
    use std::time::Duration;

    use crate::network::MTU;
    use super::DataInterface;
    use async_std;

    #[test]
    fn test_data_interface() {
        async_std::task::spawn(async {
            std::thread::sleep(Duration::from_secs(10));
            
            let mut buff = [0u8; MTU];
            for i in 0..MTU {
                buff[i] = i as u8;
            }

            let mut ds = DataInterface::with_datagram(buff, MTU);

            loop {
                let mut bbb = [0u8; 20];

                let r = ds.read_exact(&mut bbb).await;

                if r.is_err() {
                    println!("finished");
                    break;
                }
            }
        });

        println!("1234567");

        std::thread::sleep(Duration::from_secs(10000000000000000000));

    }

}
