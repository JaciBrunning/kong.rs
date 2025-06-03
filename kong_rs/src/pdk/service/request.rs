use std::collections::BTreeMap;

use http::HeaderMap;
use kong_rs_protos::Kv;
use prost_types::ListValue;
use strum::{EnumString, IntoStaticStr};

use crate::{pdk::Value, stream::Stream, KongResult};

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.service.request.set_scheme")]
  SetScheme,
  #[strum(serialize = "kong.service.request.set_path")]
  SetPath,
  #[strum(serialize = "kong.service.request.set_raw_query")]
  SetRawQuery,
  #[strum(serialize = "kong.service.request.set_method")]
  SetMethod,
  #[strum(serialize = "kong.service.request.set_query")]
  SetQuery,
  #[strum(serialize = "kong.service.request.set_header")]
  SetHeader,
  #[strum(serialize = "kong.service.request.add_header")]
  AddHeader,
  #[strum(serialize = "kong.service.request.clear_header")]
  ClearHeader,
  #[strum(serialize = "kong.service.request.set_headers")]
  SetHeaders,
  #[strum(serialize = "kong.service.request.set_raw_body")]
  SetRawBody,
}

#[derive(Clone)]
pub struct ServiceRequestPDK {
  stream: Stream,
}

impl ServiceRequestPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn set_scheme<S: Into<String>>(&self, scheme: S) -> KongResult<()> {
    self.stream.send_string(Methods::SetScheme.into(), scheme.into()).await
  }

  pub async fn set_path<S: Into<String>>(&self, path: S) -> KongResult<()> {
    self.stream.send_string(Methods::SetPath.into(), path.into()).await
  }

  pub async fn set_raw_query<S: Into<String>>(&self, query: S) -> KongResult<()> {
    self.stream.send_string(Methods::SetRawQuery.into(), query.into()).await
  }

  pub async fn set_method<S: Into<String>>(&self, method: S) -> KongResult<()> {
    self.stream.send_string(Methods::SetMethod.into(), method.into()).await
  }

  pub async fn set_query<S: Into<String>>(&self, query: BTreeMap<String, Value>) -> KongResult<()> {
    self.stream.ask_message_with_args(Methods::SetQuery.into(), &prost_types::Struct {
      fields: query.into_iter().map(|(k, v)| (k, prost_types::Value { kind: Some(v.into()) })).collect()
    }).await
  }

  pub async fn set_header(&self, name: &str, value: &str) -> KongResult<()> {
    self.stream.ask(Methods::SetHeader.into(), &Kv {
      k: name.to_owned(),
      v: Some(prost_types::Value { kind: Some(prost_types::value::Kind::StringValue(value.to_owned())) })
    }).await
  }

  pub async fn add_header(&self, name: &str, value: &str) -> KongResult<()> {
    self.stream.ask(Methods::AddHeader.into(), &Kv {
      k: name.to_owned(),
      v: Some(prost_types::Value { kind: Some(prost_types::value::Kind::StringValue(value.to_owned())) })
    }).await
  }

  pub async fn clear_header(&self, name: &str) -> KongResult<()> {
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

  pub async fn set_headers(&self, headers: HeaderMap) -> KongResult<()> {
    let s = Self::headers_to_struct(headers);
    self.stream.ask(Methods::SetHeaders.into(), &s).await
  }

  pub async fn set_body(&self, body: Vec<u8>) -> KongResult<()> {
    let bs = kong_rs_protos::ByteString { v: body };
    self.stream.ask(Methods::SetRawBody.into(), &bs).await
  }
}
