use http::HeaderMap;
use strum::{EnumString, IntoStaticStr};

use crate::{stream::Stream, KongResult};

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.service.response.get_status")]
  GetStatus,
  #[strum(serialize = "kong.service.response.get_header")]
  GetHeader,
  #[strum(serialize = "kong.service.response.get_headers")]
  GetHeaders,
  #[strum(serialize = "kong.service.response.get_raw_body")]
  GetRawBody,
}

#[derive(Clone)]
pub struct ServiceResponsePDK {
  stream: Stream,
}

impl ServiceResponsePDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_status(&self) -> KongResult<usize> {
    self.stream
      .ask_int(Methods::GetStatus.into())
      .await
      .map(|port| port as usize)
  }

  pub async fn get_header(&self, name: String) -> KongResult<String> {
    self.stream
      .ask_string_with_args(Methods::GetHeader.into(), &kong_rs_protos::String { v: name })
      .await
  }

  pub async fn get_headers(&self, max_headers: Option<usize>) -> KongResult<HeaderMap> {
    let max_headers = max_headers.unwrap_or(100);
    let headers: prost_types::Struct = self.stream.ask_message_with_args(
      Methods::GetHeaders.into(),
      &kong_rs_protos::Int { v: max_headers as i32 }
    ).await?;
    self.stream.unwrap_headers(headers)
  }

  pub async fn get_raw_body(&self) -> KongResult<Vec<u8>> {
    let body: kong_rs_protos::ByteString = self.stream.ask_message(Methods::GetRawBody.into()).await?;
    Ok(body.v)
  }
}
