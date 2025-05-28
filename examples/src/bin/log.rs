use http::Response;
use kong_rs::{pdk::{Value, Pdk}, plugin::{self, Phase, Plugin, PluginConfig, PluginFactory}, server::PluginServerBroker};
use serde_json::json;

struct LogPlugin {

}

#[derive(serde::Serialize, serde::Deserialize)]
struct LogPluginConfig { }

impl PluginConfig for LogPluginConfig {
  fn schema_fields() -> serde_json::Value {
    serde_json::to_value(json!([{
      "config": {
        "type": "record",
        "fields": [{
          "my_field": { "type": "string", "required": true }
        }]
      }
    }])).unwrap()
  }
}

#[async_trait::async_trait]
impl Plugin for LogPlugin {
  type Config = LogPluginConfig;
  const NAME: &str = "log_plugin";
  const VERSION: &str = "0.1.1";
  const PRIORITY: i32 = 10;
  const PHASES: &[Phase] = &[Phase::Access];

  async fn access(&self, pdk: &Pdk) -> plugin::Result<Vec<u8>> {

    let inner: anyhow::Result<()> = async move {
      pdk.log().err("Oh no! Anyway...").await?;
      pdk.log().err(format!("Route: {}", pdk.router().get_route().await?.name)).await?;
      Ok(())
    }.await;

    inner.unwrap();
    Ok(None)

    // Err(http::response::Builder::new().status(400).body("Uh oh!".as_bytes().to_vec()).unwrap())
    // Ok(None)
  }
}

struct LogPluginFactory {}

#[async_trait::async_trait]
impl PluginFactory for LogPluginFactory {
  type Plugin = LogPlugin;

  async fn new(&self, config_data: &str) -> Self::Plugin {
    println!("Data: {}", config_data);
    LogPlugin {  } 
  }
}

#[tokio::main]
async fn main() {
  let broker = PluginServerBroker::new();
  broker.register(LogPluginFactory {}).await;
  broker.run(std::env::args()).await.unwrap();
}
