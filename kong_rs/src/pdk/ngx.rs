use strum::{EnumString, IntoStaticStr};

use crate::{stream::Stream, KongResult};

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.ngx.get_var")]
  GetVar,
}

#[derive(Clone)]
pub struct NgxPDK {
  stream: Stream
}

impl NgxPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_var<K: Into<String>>(&self, key: K) -> KongResult<String> {
    self.stream.ask_string_with_args(Methods::GetVar.into(), &kong_rs_protos::String { v: key.into() }).await
  }
}
