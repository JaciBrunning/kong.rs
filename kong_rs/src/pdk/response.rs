use http::HeaderMap;
use kong_rs_protos::{ExitArgs, Kv};
use prost_types::ListValue;
use strum::{EnumString, IntoStaticStr};

use crate::stream::Stream;

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.response.get_status")]
  GetStatus,
  #[strum(serialize = "kong.response.get_header")]
  GetHeader,
  #[strum(serialize = "kong.response.get_headers")]
  GetHeaders,
  #[strum(serialize = "kong.response.get_source")]
  GetSource,
  #[strum(serialize = "kong.response.set_status")]
  SetStatus,
  #[strum(serialize = "kong.response.set_header")]
  SetHeader,
  #[strum(serialize = "kong.response.add_header")]
  AddHeader,
  #[strum(serialize = "kong.response.clear_header")]
  ClearHeader,
  #[strum(serialize = "kong.response.set_headers")]
  SetHeaders,
  #[strum(serialize = "kong.response.exit")]
  Exit,
}

#[derive(Clone)]
pub struct ResponsePDK {
  stream: Stream,
}

impl ResponsePDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_status(&self) -> anyhow::Result<usize> {
    self.stream
      .ask_int(Methods::GetStatus.into())
      .await
      .map(|port| port as usize)
  }
  
  pub async fn get_header(&self, name: String) -> anyhow::Result<String> {
    self.stream
      .ask_string_with_args(Methods::GetHeader.into(), &kong_rs_protos::String { v: name })
      .await
  }

  pub async fn get_headers(&self, max_headers: Option<usize>) -> anyhow::Result<HeaderMap> {
    let max_headers = max_headers.unwrap_or(100);
    let headers: prost_types::Struct = self.stream.ask_message_with_args(
      Methods::GetHeaders.into(),
      &kong_rs_protos::Int { v: max_headers as i32 }
    ).await?;
    self.stream.unwrap_headers(headers)
  }

  pub async fn get_source(&self) -> anyhow::Result<String> {
    self.stream.ask_string(Methods::GetStatus.into()).await
  }

  pub async fn set_status(&self, status: usize) -> anyhow::Result<()> {
    self.stream.send_int(Methods::SetStatus.into(), status as i32).await
  }

  pub async fn set_header(&self, name: &str, value: &str) -> anyhow::Result<()> {
    self.stream.ask(Methods::SetHeader.into(), &Kv {
      k: name.to_owned(),
      v: Some(prost_types::Value { kind: Some(prost_types::value::Kind::StringValue(value.to_owned())) })
    }).await
  }

  pub async fn add_header(&self, name: &str, value: &str) -> anyhow::Result<()> {
    self.stream.ask(Methods::AddHeader.into(), &Kv {
      k: name.to_owned(),
      v: Some(prost_types::Value { kind: Some(prost_types::value::Kind::StringValue(value.to_owned())) })
    }).await
  }

  pub async fn clear_header(&self, name: &str) -> anyhow::Result<()> {
    self.stream.ask(Methods::ClearHeader.into(), &kong_rs_protos::String { v: name.to_owned() }).await
  }

  fn headers_to_struct(headers: HeaderMap) -> prost_types::Struct {
    let mut s = prost_types::Struct { ..Default::default() };

    for key in headers.keys() {
      let values = headers.get_all(key);
      s.fields.insert(
        key.to_string(),
        prost_types::Value {
          kind: Some(prost_types::value::Kind::ListValue(ListValue {
            values: values.into_iter().map(|x| prost_types::Value {
              kind: Some(prost_types::value::Kind::StringValue(std::str::from_utf8(x.as_bytes()).unwrap_or("").to_owned()))
            }).collect()
          }))
        }
      );
    }

    s
  }

  pub async fn set_headers(&self, headers: HeaderMap) -> anyhow::Result<()> {
    let s = Self::headers_to_struct(headers);
    self.stream.ask(Methods::SetHeaders.into(), &s).await
  }

  pub async fn exit(&self, status: usize, body: Vec<u8>, headers: Option<HeaderMap>) -> anyhow::Result<()> {
    let exit_args = ExitArgs { status: status as i32, body, headers: headers.map(Self::headers_to_struct) };
    self.stream.ask(Methods::Exit.into(), &exit_args).await
  }
}
