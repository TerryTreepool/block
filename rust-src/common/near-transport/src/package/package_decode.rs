
use async_std::io::{BufReader, Read, ReadExt};

use log::error;
use near_base::{Deserialize, ErrorCode, NearError, NearResult, RawFixedBytes};

use crate::network::{DataContext, MTU};

use super::{PackageHeader, PackageHeaderExt};

pub async fn decode_package<IO>(io: IO) -> NearResult<DataContext>
where IO: Read + Unpin {
    let mut recv_buf = [0u8; MTU];
    let mut reader = BufReader::new(io);

    // recv and parse package head
    let (packet_head, remain_buf) = {
        let packet_header_len = PackageHeader::raw_bytes();
        let header_buf = &mut recv_buf[..packet_header_len];
        reader.read_exact(header_buf).await?;

        let (header, _) = PackageHeader::deserialize(header_buf)?;

        (header, &mut recv_buf[packet_header_len..])
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
        reader.read_exact(body_buf)
            .await
            .map_err(|err| {
                let error_message = format!("failed recv pakcet body with errno: {}", err);
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
