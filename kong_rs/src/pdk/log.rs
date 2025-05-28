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

  pub async fn alert<T: Into<String>>(&mut self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Alert, args.into()).await
  }

  pub async fn crit<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Crit, args.into()).await
  }

  pub async fn err<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Error, args.into()).await
  }

  pub async fn warn<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Warn, args.into()).await
  }

  pub async fn notice<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Notice, args.into()).await
  }

  pub async fn info<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Info, args.into()).await
  }

  pub async fn debug<T: Into<String>>(&self, args: T) -> anyhow::Result<()> {
    self.do_log(Methods::Debug, args.into()).await
  }

  pub async fn serialize(&self) -> anyhow::Result<String> {
    self.stream.ask_string(Methods::Serialize.into()).await
  }
}