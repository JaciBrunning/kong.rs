use kong_rs::{ok_or_internal_error, KongError, Pdk, Phase, Plugin, PluginFactory, PluginResult, PluginServerBroker};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, kong_rs::PluginConfig)]
enum MyEnum {
  Test1,
  Test2,
  Test3
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, kong_rs::PluginConfig)]
struct InnerConfig {
  a: String,
  b: Option<String>,
  c: MyEnum
}

impl Default for InnerConfig {
  fn default() -> Self {
    Self {
      a: "Test".to_owned(),
      b: None,
      c: MyEnum::Test2
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, kong_rs::PluginConfig)]
struct LogPluginConfig {
  my_field: String,
  my_other_field: Vec<isize>,
  inner: InnerConfig
}

impl Default for LogPluginConfig {
  fn default() -> Self {
    Self {
      my_field: "Hello World".to_owned(),
      my_other_field: vec![42, 69, 420],
      inner: InnerConfig::default()
    }
  }
}

struct LogPlugin { }

#[async_trait::async_trait]
impl Plugin for LogPlugin {
  type Config = LogPluginConfig;
  const NAME: &str = "log_plugin";
  const VERSION: &str = "0.1.1";
  const PRIORITY: i32 = 10;
  const PHASES: &[Phase] = &[Phase::Access];

  async fn access(&self, pdk: &Pdk) -> PluginResult<Vec<u8>> {
    ok_or_internal_error(async move {
      pdk.log().err("Oh no! Anyway...").await?;
      pdk.log().err(format!("Route: {}", pdk.router().get_route().await?.name)).await?;
      Ok(())
    }.await)?;

    Ok(None)
  }

  fn default_config() -> Self::Config { Self::Config::default() }
}

struct LogPluginFactory {}

#[async_trait::async_trait]
impl PluginFactory for LogPluginFactory {
  type Plugin = LogPlugin;

  async fn new(&self, config_data: &str) -> Self::Plugin {
    println!("Data: {:?}", serde_json::from_str::<'_, LogPluginConfig>(config_data).unwrap());
    LogPlugin {  } 
  }
}

#[tokio::main]
async fn main() {
  let broker = PluginServerBroker::new();
  broker.register(LogPluginFactory {}).await;
  broker.run(std::env::args()).await.unwrap();
}
