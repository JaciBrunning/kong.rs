use log::LogPDK;
use request::RequestPDK;
use response::ResponsePDK;

use crate::stream::Stream;

pub mod log;
pub mod request;
pub mod response;

pub struct Pdk {
  stream: Stream
}

impl Pdk {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub fn log(&self) -> LogPDK {
    LogPDK::new(self.stream.clone())
  }

  pub fn request(&self) -> RequestPDK {
    RequestPDK::new(self.stream.clone())
  }

  pub fn response(&self) -> ResponsePDK {
    ResponsePDK::new(self.stream.clone())
  }
}
