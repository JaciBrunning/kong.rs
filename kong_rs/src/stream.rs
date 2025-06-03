use std::{str::FromStr, sync::Arc};

use http::{HeaderMap, HeaderName, HeaderValue};
use prost::Message;

use crate::KongResult;

// From https://github.com/jgramoll/kong-rust-pdk, slightly adjusted.

#[derive(Clone)]
pub struct Stream(pub Arc<tokio::net::UnixStream>);

impl Stream {
  pub fn new(stream: tokio::net::UnixStream) -> Self {
    Self(Arc::new(stream))
  }
}

impl Stream {
  pub async fn write_method(&self, method: &str) -> KongResult<usize> {
    let res1 = self.write_frame(method.as_bytes()).await?;
    // empty frame for 0 args
    let res2 = self.write_frame(&[]).await?;

    Ok(res1 + res2)
  }

  async fn write_method_with_args<T: Message>(
    &self,
    method: &str,
    args: &T,
  ) -> KongResult<usize> {
    let res1 = self.write_frame(method.as_bytes()).await?;
    let res2 = self.write_frame(&args.encode_to_vec()).await?;

    Ok(res1 + res2)
  }

  pub async fn ask<T: prost::Message>(&self, method: &str, args: &T) -> KongResult<()> {
    self.write_method_with_args(method, args).await?;
    self.read_frame().await?;
    Ok(())
  }

  pub async fn ask_message_with_args<T: prost::Message, R: prost::Message + Default>(
    &self,
    method: &str,
    args: &T,
  ) -> KongResult<R> {
    self.write_method_with_args(method, args).await?;
    let out = self.read_message::<R>().await?;
    Ok(out)
  }

  pub async fn ask_message<R: prost::Message + Default>(
    &self,
    method: &str,
  ) -> KongResult<R> {
    self.write_method(method).await?;
    let out = self.read_message::<R>().await?;
    Ok(out)
  }

  #[allow(dead_code)]
  pub async fn send_string(&self, method: &str, v: String) -> KongResult<()> {
    self.ask(method, &kong_rs_protos::String { v }).await
  }

  pub async fn send_int(&self, method: &str, v: i32) -> KongResult<()> {
    self.ask(method, &kong_rs_protos::Int { v }).await
  }

  pub async fn ask_string(&self, method: &str) -> KongResult<String> {
    self.write_method(method).await?;
    let s = self.read_message::<kong_rs_protos::String>().await?;
    Ok(s.v)
  }

  pub async fn ask_string_with_args<T: prost::Message>(
    &self,
    method: &str,
    args: &T,
  ) -> KongResult<String> {
    self.write_method_with_args(method, args).await?;
    let s = self.read_message::<kong_rs_protos::String>().await?;
    Ok(s.v)
  }

  pub async fn ask_int(&self, method: &str) -> KongResult<i32> {
    self.write_method(method).await?;
    let s = self.read_message::<kong_rs_protos::Int>().await?;
    Ok(s.v)
  }

  #[allow(dead_code)]
  pub async fn ask_int_with_args<T: prost::Message>(
    &self,
    method: &str,
    args: &T,
  ) -> KongResult<i32> {
    self.write_method_with_args(method, args).await?;
    let s = self.read_message::<kong_rs_protos::Int>().await?;
    Ok(s.v)
  }

  pub async fn ask_number(&self, method: &str) -> KongResult<f64> {
    self.write_method(method).await?;
    let s = self.read_message::<kong_rs_protos::Number>().await?;
    Ok(s.v)
  }

  fn unwrap_single_header(name: &HeaderName, kind: prost_types::value::Kind, ret: &mut HeaderMap) -> KongResult<()> {
    match kind {
      prost_types::value::Kind::NullValue(_) => (),
      prost_types::value::Kind::NumberValue(n) => {
        ret.append(name, HeaderValue::from_str(&n.to_string())?);
      }
      prost_types::value::Kind::StringValue(str) => {
        ret.append(name, HeaderValue::from_str(&str)?);
      },
      prost_types::value::Kind::BoolValue(b) => {
        ret.append(name, HeaderValue::from_str(&b.to_string())?);
      }
      prost_types::value::Kind::StructValue(_) => {
        // TODO: How do?
      },
      prost_types::value::Kind::ListValue(l) => {
        for v in l.values {
          if let Some(kind) = v.kind {
            Self::unwrap_single_header(name, kind, ret)?;
          }
        }
      }
    }
    Ok(())
  }

  pub fn unwrap_headers(&self, st: prost_types::Struct) -> KongResult<HeaderMap> {
    let mut ret = HeaderMap::default();

    for (name, v) in st.fields {
      if let Some(kind) = v.kind {
        let name = HeaderName::from_str(&name).unwrap();
        Self::unwrap_single_header(&name, kind, &mut ret)?;
      }
    }

    Ok(ret)
  }
}

impl Stream {
  // read bytes from stream to given array
  pub async fn read(&self, mut out: &mut [u8]) -> KongResult<usize> {
    loop {
      self.0.readable().await?;
      match self.0.try_read(&mut out) {
        Ok(0) => return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted).into()),
        Ok(n) => {
          if n > 0 {
            break Ok(n);
          }
        }
        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
          continue;
        }
        Err(e) => {
          break Err(e.into());
        }
      }
    }
  }

  async fn read_i32(&self) -> KongResult<i32> {
    let mut bytes = [0; 4];
    let len = self.read(&mut bytes).await?;
    debug_assert!(len == 4);
    Ok(i32::from_le_bytes(bytes))
  }

  pub async fn read_frame(&self) -> KongResult<Vec<u8>> {
    // read len + msg
    let len = self.read_i32().await? as usize;
    if len == 0 {
        return Ok(vec![]);
    }

    let mut buf = vec![0; len];
    let read_len = self.read(&mut buf).await?;
    debug_assert_eq!(read_len, len);

    let (bytes, _) = buf.split_at(read_len);
    Ok(bytes.to_vec())
  }

  pub async fn read_message<T: Message + Default>(&self) -> KongResult<T> {
    let bytes = self.read_frame().await?;
    let t = T::decode(&*bytes)?;
    Ok(t)
  }
}

impl Stream {
  async fn write(&self, buf: &[u8]) -> KongResult<usize> {
    loop {
      self.0.writable().await?;

      match self.0.try_write(buf) {
        Ok(n) => {
          break Ok(n);
        }
        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
          continue;
        }
        Err(e) => {
          break Err(e.into());
        }
      }
    }
  }

  pub async fn write_frame(&self, buf: &[u8]) -> KongResult<usize> {
    // send len + msg
    let len = buf.len();
    let res1 = self.write(&(len as u32).to_le_bytes()).await?;
    let res2 = self.write(buf).await?;

    Ok(res1 + res2)
  }

  pub async fn write_message<T: Message>(&self, msg: &T) -> KongResult<usize> {
    self.write_frame(&msg.encode_to_vec()).await
  }
}