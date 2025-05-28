use kong_rs_protos::Kv;
use strum::{EnumString, IntoStaticStr};

use crate::stream::Stream;

use super::Value;

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.ctx.shared.set")]
  SharedSet,
  #[strum(serialize = "kong.ctx.shared.get")]
  SharedGet,
  #[strum(serialize = "kong.nginx.set_ctx")]
  Set,
  #[strum(serialize = "kong.nginx.get_ctx")]
  Get,
}

#[derive(Clone)]
pub struct CtxPDK {
  stream: Stream
}

impl CtxPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn shared_set<K: Into<String>>(&self, key: K, value: Value) -> anyhow::Result<()> {
    let kind = match value {
        Value::Null => None,
        x => Some(x.into())
    };

    let kv = Kv { k: key.into(), v: Some(prost_types::Value { kind: kind }) };
    self.stream.ask_message_with_args(Methods::SharedSet.into(), &kv).await
  }

  pub async fn shared_get<K: Into<String>>(&self, key: K) -> anyhow::Result<Value> {
    let v: prost_types::Value = self.stream.ask_message_with_args(
      Methods::SharedGet.into(),
      &kong_rs_protos::String { v: key.into() }
    ).await?;

    Ok(v.kind.map(Into::into).unwrap_or(Value::Null))
  }

  pub async fn set<K: Into<String>>(&self, key: K, value: Value) -> anyhow::Result<()> {
    let kind = match value {
        Value::Null => None,
        x => Some(x.into())
    };

    let kv = Kv { k: key.into(), v: Some(prost_types::Value { kind: kind }) };
    self.stream.ask_message_with_args(Methods::Set.into(), &kv).await
  }

  pub async fn get<K: Into<String>>(&self, key: K) -> anyhow::Result<Value> {
    let v: prost_types::Value = self.stream.ask_message_with_args(
      Methods::Get.into(),
      &kong_rs_protos::String { v: key.into() }
    ).await?;

    Ok(v.kind.map(Into::into).unwrap_or(Value::Null))
  }
}
