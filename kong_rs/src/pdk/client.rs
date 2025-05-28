use kong_rs_protos::{AuthenticateArgs, AuthenticatedCredential, Consumer, ConsumerSpec};
use strum::{EnumString, IntoStaticStr};

use crate::stream::Stream;


#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.client.get_ip")]
  GetIp,
  #[strum(serialize = "kong.client.get_forwarded_ip")]
  GetForwardedIp,
  #[strum(serialize = "kong.client.get_port")]
  GetPort,
  #[strum(serialize = "kong.client.get_forwarded_port")]
  GetForwardedPort,
  #[strum(serialize = "kong.client.get_credential")]
  GetCredential,
  #[strum(serialize = "kong.client.load_consumer")]
  LoadConsumer,
  #[strum(serialize = "kong.client.get_consumer")]
  GetConsumer,
  #[strum(serialize = "kong.client.authenticate")]
  Authenticate,
  #[strum(serialize = "kong.client.get_protocol")]
  GetProtocol,
}

#[derive(Clone)]
pub struct ClientPDK {
  stream: Stream
}

impl ClientPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_ip(&self) -> anyhow::Result<String> {
    self.stream.ask_string(Methods::GetIp.into()).await
  }

  pub async fn get_forwarded_ip(&self) -> anyhow::Result<String> {
    self.stream.ask_string(Methods::GetForwardedIp.into()).await
  }

  pub async fn get_port(&self) -> anyhow::Result<usize> {
    self.stream.ask_int(Methods::GetPort.into()).await.map(|port| port as usize)
  }

  pub async fn get_forwarded_port(&self) -> anyhow::Result<usize> {
    self.stream.ask_int(Methods::GetForwardedPort.into()).await.map(|port| port as usize)
  }

  pub async fn get_credential(&self) -> anyhow::Result<AuthenticatedCredential> {
    self.stream.ask_message(Methods::GetCredential.into()).await
  }

  pub async fn load_consumer(&self, consumer: ConsumerSpec) -> anyhow::Result<Consumer> {
    self.stream.ask_message_with_args(Methods::LoadConsumer.into(), &consumer).await
  }

  pub async fn get_consumer(&self) -> anyhow::Result<Consumer> {
    self.stream.ask_message(Methods::GetConsumer.into()).await
  }

  pub async fn authenticate(&self, auth: AuthenticateArgs) -> anyhow::Result<()> {
    self.stream.ask(Methods::Authenticate.into(), &auth).await
  }

  pub async fn get_protocol(&self, allow_terminated: bool) -> anyhow::Result<String> {
    self.stream.ask_string_with_args(Methods::GetProtocol.into(), &kong_rs_protos::Bool { v: allow_terminated }).await
  }
}