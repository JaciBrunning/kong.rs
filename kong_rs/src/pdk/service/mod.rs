use kong_rs_protos::Target;
use request::ServiceRequestPDK;
use response::ServiceResponsePDK;
use strum::{EnumString, IntoStaticStr};

use crate::{stream::Stream, KongResult};

pub mod request;
pub mod response;

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.service.set_upstream")]
  SetUpstream,
  #[strum(serialize = "kong.service.set_target")]
  SetTarget,
}

#[derive(Clone)]
pub struct ServicePDK {
  stream: Stream,

  request: ServiceRequestPDK,
  response: ServiceResponsePDK
}

impl ServicePDK {
  pub fn new(stream: Stream) -> Self {
    Self {
      stream: stream.clone(),
      request: ServiceRequestPDK::new(stream.clone()),
      response: ServiceResponsePDK::new(stream.clone()),
    }
  }

  pub async fn set_upstream<A: Into<String>>(&self, addr: A) -> KongResult<bool> {
    let r: kong_rs_protos::Bool = self.stream.ask_message_with_args(Methods::SetUpstream.into(), &kong_rs_protos::String { v: addr.into() }).await?;
    Ok(r.v)
  }

  pub async fn set_target<H: Into<String>>(&self, host: H, port: usize) -> KongResult<()> {
    self.stream.ask_message_with_args(Methods::SetTarget.into(), &Target { host: host.into(), port: port as i32 }).await
  }

  pub fn request(&self) -> &ServiceRequestPDK {
    &self.request
  }

  pub fn response(&self) -> &ServiceResponsePDK {
    &self.response
  }
}
