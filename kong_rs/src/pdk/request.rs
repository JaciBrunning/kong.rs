use http::HeaderMap;
use kong_rs_protos::RawBodyResult;
use strum::{EnumString, IntoStaticStr};

use crate::{stream::Stream, KongError, KongResult};

pub enum Body {
  Content(Vec<u8>),
  Path(String),
  Empty
}

#[derive(Debug, PartialEq, IntoStaticStr, EnumString)]
pub(crate) enum Methods {
  #[strum(serialize = "kong.request.get_scheme")]
  GetScheme,
  #[strum(serialize = "kong.request.get_host")]
  GetHost,
  #[strum(serialize = "kong.request.get_port")]
  GetPort,
  #[strum(serialize = "kong.request.get_forwarded_scheme")]
  GetForwardedScheme,
  #[strum(serialize = "kong.request.get_forwarded_host")]
  GetForwardedHost,
  #[strum(serialize = "kong.request.get_forwarded_port")]
  GetForwardedPort,
  #[strum(serialize = "kong.request.get_http_version")]
  GetHttpVersion,
  #[strum(serialize = "kong.request.get_method")]
  GetMethod,
  #[strum(serialize = "kong.request.get_path")]
  GetPath,
  #[strum(serialize = "kong.request.get_path_with_query")]
  GetPathWithQuery,
  #[strum(serialize = "kong.request.get_raw_query")]
  GetRawQuery,
  #[strum(serialize = "kong.request.get_query_arg")]
  GetQueryArg,
  #[strum(serialize = "kong.request.get_query")]
  GetQuery,
  #[strum(serialize = "kong.request.get_header")]
  GetHeader,
  #[strum(serialize = "kong.request.get_headers")]
  GetHeaders,
  #[strum(serialize = "kong.request.get_raw_body")]
  GetRawBody,
}

#[derive(Clone)]
pub struct RequestPDK {
  stream: Stream,
}

impl RequestPDK {
  pub fn new(stream: Stream) -> Self {
    Self { stream }
  }

  pub async fn get_scheme(&self) -> KongResult<String> {
    self.stream.ask_string(Methods::GetScheme.into()).await
  }

  pub async fn get_host(&self) -> KongResult<String> {
    self.stream.ask_string(Methods::GetHost.into()).await
  }

  pub async fn get_port(&self) -> KongResult<usize> {
    self.stream
      .ask_int(Methods::GetPort.into())
      .await
      .map(|port| port as usize)
  }

  pub async fn get_forwarded_scheme(&self) -> KongResult<String> {
    self.stream
      .ask_string(Methods::GetForwardedScheme.into())
      .await
  }

  pub async fn get_forwarded_host(&self) -> KongResult<String> {
    self.stream
      .ask_string(Methods::GetForwardedHost.into())
      .await
  }

  pub async fn get_forwarded_port(&self) -> KongResult<usize> {
    self.stream
      .ask_int(Methods::GetForwardedPort.into())
      .await
      .map(|port| port as usize)
  }

  pub async fn get_http_version(&self) -> KongResult<f64> {
    self.stream
      .ask_number(Methods::GetForwardedPort.into())
      .await
  }

  pub async fn get_method(&self) -> KongResult<String> {
    self.stream.ask_string(Methods::GetMethod.into()).await
  }

  pub async fn get_path(&self) -> KongResult<String> {
    self.stream.ask_string(Methods::GetPath.into()).await
  }

  pub async fn get_path_with_query(&self) -> KongResult<String> {
    self.stream
      .ask_string(Methods::GetPathWithQuery.into())
      .await
  }

  pub async fn get_raw_query(&self) -> KongResult<String> {
    self.stream.ask_string(Methods::GetRawQuery.into()).await
  }

  pub async fn get_query_arg(&self, name: String) -> KongResult<String> {
    self.stream
      .ask_string_with_args(Methods::GetQueryArg.into(), &kong_rs_protos::String { v: name })
      .await
  }

  pub async fn get_query(&self, max_args: Option<usize>) -> KongResult<HeaderMap> {
    let max_args = max_args.unwrap_or(100);
    let headers: prost_types::Struct = self.stream.ask_message_with_args(
      Methods::GetQuery.into(),
      &kong_rs_protos::Int { v: max_args as i32 }
    ).await?;
    self.stream.unwrap_headers(headers)
  }

  pub async fn get_header(&self, name: String) -> KongResult<String> {
    self.stream
      .ask_string_with_args(Methods::GetHeader.into(), &kong_rs_protos::String { v: name })
      .await
  }

  pub async fn get_headers(&self, max_headers: Option<usize>) -> KongResult<HeaderMap> {
    let max_headers = max_headers.unwrap_or(100);
    let headers: prost_types::Struct = self.stream.ask_message_with_args(
      Methods::GetHeaders.into(),
      &kong_rs_protos::Int { v: max_headers as i32 }
    ).await?;
    self.stream.unwrap_headers(headers)
  }

  pub async fn get_raw_body(&self) -> KongResult<Body> {
    let body: RawBodyResult = self.stream.ask_message(Methods::GetRawBody.into()).await?;
    match body.kind {
      Some(kind) => match kind {
        kong_rs_protos::raw_body_result::Kind::Content(items) => {
          Ok(Body::Content(items))
        },
        kong_rs_protos::raw_body_result::Kind::BodyFilepath(path) => {
          Ok(Body::Path(path))
        },
        kong_rs_protos::raw_body_result::Kind::Error(err) => {
          Err(KongError::BodyError(format!("Body Error: {}", err)))
        },
      },
      None => Ok(Body::Empty),
    }
  }
}
