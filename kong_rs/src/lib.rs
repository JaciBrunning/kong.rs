pub mod config;
pub mod pdk;
pub mod plugin;
pub mod server;
pub mod stream;

use http::{Response, StatusCode};
pub use kong_rs_macros::PluginConfig;

pub use pdk::Pdk;
pub use plugin::{Phase, Plugin, PluginFactory, PluginResult};
pub use server::PluginServerBroker;

#[derive(Debug)]
pub enum KongError {
  IOError(std::io::Error),
  ProtobufDecodeError(prost::DecodeError),
  HeaderParseError(http::header::InvalidHeaderValue),
  LaunchError(String),
  SerdeError(serde_json::Error),
  EncodingError(std::str::Utf8Error),
  InvalidValueError(String),
  BodyError(String)
}

impl From<std::io::Error> for KongError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}

impl From<prost::DecodeError> for KongError {
  fn from(value: prost::DecodeError) -> Self {
    Self::ProtobufDecodeError(value)
  }
}

impl From<http::header::InvalidHeaderValue> for KongError {
  fn from(value: http::header::InvalidHeaderValue) -> Self {
    Self::HeaderParseError(value)
  }
}

impl From<serde_json::Error> for KongError {
  fn from(value: serde_json::Error) -> Self {
    Self::SerdeError(value)
  }
}

impl From<std::str::Utf8Error> for KongError {
  fn from(value: std::str::Utf8Error) -> Self {
    Self::EncodingError(value)
  }
}

pub type KongResult<T> = std::result::Result<T, KongError>;

impl KongError {
  pub fn to_internal_error(self) -> Response<Vec<u8>> {
    let mut response = Response::new("The server encountered an unexpected error!".as_bytes().to_vec());
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response
  }
}

pub fn ok_or_internal_error<T>(result: KongResult<T>) -> std::result::Result<T, Response<Vec<u8>>> {
  match result {
    Ok(ok) => Ok(ok),
    Err(e) => Err(e.to_internal_error()),
  }
}
