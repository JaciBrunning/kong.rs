use kong_rs_protos::{Route, Service};
use strum::{EnumString, IntoStaticStr};

use crate::{stream::Stream, KongResult};

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.router.get_route")]
  GetRoute,
  #[strum(serialize = "kong.router.get_service")]
  GetService,
}

#[derive(Clone)]
pub struct RouterPDK {
  stream: Stream
}

impl RouterPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_route(&self) -> KongResult<Route> {
    self.stream.ask_message(Methods::GetRoute.into()).await
  }

  pub async fn get_service(&self) -> KongResult<Service> {
    self.stream.ask_message(Methods::GetService.into()).await
  }
}
