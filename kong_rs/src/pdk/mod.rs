use std::collections::BTreeMap;

use client::ClientPDK;
use ctx::CtxPDK;
use log::LogPDK;
use ngx::NgxPDK;
use request::RequestPDK;
use response::ResponsePDK;
use router::RouterPDK;
use service::ServicePDK;

use crate::stream::Stream;

pub mod client;
pub mod ctx;
pub mod log;
pub mod ngx;
pub mod request;
pub mod response;
pub mod router;
pub mod service;

#[derive(Debug, Clone)]
pub enum Value {
  Null,
  Number(f64),
  String(String),
  Bool(bool),
  Struct(BTreeMap<String, Self>),
  List(Vec<Self>)
}

impl From<prost_types::value::Kind> for Value {
  fn from(value: prost_types::value::Kind) -> Self {
    match value {
      prost_types::value::Kind::NullValue(_) => Value::Null,
      prost_types::value::Kind::NumberValue(number) => Value::Number(number),
      prost_types::value::Kind::StringValue(str) => Value::String(str),
      prost_types::value::Kind::BoolValue(b) => Value::Bool(b),
      prost_types::value::Kind::StructValue(struct_val) => Value::Struct(
        struct_val.fields.into_iter().map(|(k, v)| (k, v.kind.map(|x| x.into()).unwrap_or(Value::Null))).collect()
      ),
      prost_types::value::Kind::ListValue(list_val) => Value::List(
        list_val.values.into_iter().map(|v| v.kind.map(|x| x.into()).unwrap_or(Value::Null)).collect()
      ),
    }
  }
}

impl Into<prost_types::value::Kind> for Value {
  fn into(self) -> prost_types::value::Kind {
    match self {
      Value::Null => prost_types::value::Kind::NullValue(0),
      Value::Number(number) => prost_types::value::Kind::NumberValue(number),
      Value::String(str) => prost_types::value::Kind::StringValue(str),
      Value::Bool(b) => prost_types::value::Kind::BoolValue(b),
      Value::Struct(fields) => prost_types::value::Kind::StructValue(
        prost_types::Struct { fields: fields.into_iter().map(|(k, v)| (k, prost_types::Value { kind: Some(v.into()) })).collect() }
      ),
      Value::List(values) => prost_types::value::Kind::ListValue(
        prost_types::ListValue { values: values.into_iter().map(|v| prost_types::Value { kind: Some(v.into()) }).collect() }
      ),
    }
  }
}


pub struct Pdk {
  client: ClientPDK,
  ctx: CtxPDK,
  log: LogPDK,
  ngx: NgxPDK,
  request: RequestPDK,
  response: ResponsePDK,
  router: RouterPDK,
  service: ServicePDK
}

impl Pdk {
  pub fn new(stream: Stream) -> Self {
    Self {
      client: ClientPDK::new(stream.clone()),
      ctx: CtxPDK::new(stream.clone()),
      log: LogPDK::new(stream.clone()),
      ngx: NgxPDK::new(stream.clone()),
      request: RequestPDK::new(stream.clone()),
      response: ResponsePDK::new(stream.clone()),
      router: RouterPDK::new(stream.clone()),
      service: ServicePDK::new(stream.clone()),
    }
  }

  pub fn client(&self) -> &ClientPDK {
    &self.client
  }

  pub fn ctx(&self) -> &CtxPDK {
    &self.ctx
  }

  pub fn log(&self) -> &LogPDK {
    &self.log
  }

  pub fn ngx(&self) -> &NgxPDK {
    &self.ngx
  }

  pub fn request(&self) -> &RequestPDK {
    &self.request
  }

  pub fn response(&self) -> &ResponsePDK {
    &self.response
  }

  pub fn router(&self) -> &RouterPDK {
    &self.router
  }

  pub fn service(&self) -> &ServicePDK {
    &self.service
  }
}
