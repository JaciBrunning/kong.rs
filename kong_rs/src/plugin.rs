use http::Response;

use crate::pdk::Pdk;

pub type Result<T> = std::result::Result<Option<Response<T>>, Response<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
  Access
}

impl Into<&'static str> for Phase {
  fn into(self) -> &'static str {
    match self {
      Phase::Access => "access",
    }
  }
}

impl TryFrom<&str> for Phase {
  type Error = ();

  fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
    if value == "access" {
      Ok(Phase::Access)
    } else {
      Err(())
    }
  }
}

#[async_trait::async_trait]
pub trait Plugin : Send + Sync {
  type Config : PluginConfig;

  const NAME: &str;
  const VERSION: &str;
  const PRIORITY: i32;

  const PHASES: &[Phase];
  
  async fn access(&self, pdk: &Pdk) -> Result<Vec<u8>>;
}

#[async_trait::async_trait]
pub trait ErasedPlugin {
  async fn _call_phase(&self, phase: &Phase, pdk: &Pdk);
  fn name(&self) -> String;
}

#[async_trait::async_trait]
impl<P: Plugin> ErasedPlugin for P {
  async fn _call_phase(&self, phase: &Phase, pdk: &Pdk) {
    let result = match phase {
      Phase::Access => self.access(pdk).await,
    };

    let result = match result {
      Ok(Some(ok_response)) => {
        pdk.response().exit(ok_response.status().as_u16() as usize, ok_response.body().to_vec(), Some(ok_response.headers().clone())).await
      },
      Ok(None) => { Ok(()) },
      Err(err_response) => {
        pdk.response().exit(err_response.status().as_u16() as usize, err_response.body().to_vec(), Some(err_response.headers().clone())).await
      },
    };

    result.expect("Unknown error during early exit. Killing the process as a precaution.");
  }

  fn name(&self) -> String {
    Self::NAME.to_owned()
  }
}

pub trait PluginConfig {
  fn schema_fields() -> serde_json::Value;
}

pub struct PluginInfo {
  pub name: String,
  pub phases: Vec<Phase>,
  pub version: String,
  pub priority: i32,
  pub schema: serde_json::Value,
}

#[async_trait::async_trait]
pub trait PluginFactory {
  type Plugin: Plugin + 'static;
  async fn new(&self, config_data: &str) -> Self::Plugin;
}

#[async_trait::async_trait]
pub trait ErasedPluginFactory: Send + Sync {
  async fn new(&self, config_data: &str) -> Box<dyn ErasedPlugin + Send + Sync>;
  fn get_info(&self) -> PluginInfo;
}

#[async_trait::async_trait]
impl<F: PluginFactory + Send + Sync> ErasedPluginFactory for F {
  async fn new(&self, config_data: &str) -> Box<dyn ErasedPlugin + Send + Sync> {
    Box::new(<F as PluginFactory>::new(self, config_data).await)
  }

  fn get_info(&self) -> PluginInfo {
    PluginInfo {
      name: F::Plugin::NAME.to_owned(),
      phases: F::Plugin::PHASES.to_vec(),
      version: F::Plugin::VERSION.to_owned(),
      priority: F::Plugin::PRIORITY,
      schema: <F::Plugin as Plugin>::Config::schema_fields()
    }
  }
}
