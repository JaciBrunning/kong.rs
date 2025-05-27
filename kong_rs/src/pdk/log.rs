use strum::{EnumString, IntoStaticStr};

use crate::stream::Stream;

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.log.alert")]
  Alert,
  #[strum(serialize = "kong.log.crit")]
  Crit,
  #[strum(serialize = "kong.log.err")]
  Error,
  #[strum(serialize = "kong.log.warn")]
  Warn,
  #[strum(serialize = "kong.log.notice")]
  Notice,
  #[strum(serialize = "kong.log.info")]
  Info,
  #[strum(serialize = "kong.log.debug")]
  Debug,
  #[strum(serialize = "kong.log.serialize")]
  Serialize,
}

#[derive(Clone)]
pub struct LogPDK {
  stream: Stream
}

impl LogPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  async fn do_log(&self, method: Methods, args: String) -> anyhow::Result<()> {
    self.stream.ask(method.into(), &prost_types::ListValue {
      values: vec![prost_types::Value { kind: Some(prost_types::value::Kind::StringValue(args)) }]
    }).await
  }

  pub async fn alert(&mut self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Alert, args).await
  }

  pub async fn crit(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Crit, args).await
  }

  pub async fn err(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Error, args).await
  }

  pub async fn warn(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Warn, args).await
  }

  pub async fn notice(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Notice, args).await
  }

  pub async fn info(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Info, args).await
  }

  pub async fn debug(&self, args: String) -> anyhow::Result<()> {
    self.do_log(Methods::Debug, args).await
  }

  pub async fn serialize(&self) -> anyhow::Result<String> {
    self.stream.ask_string(Methods::Serialize.into()).await
  }
}