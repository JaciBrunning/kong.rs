use std::{collections::HashMap, path::Path, sync::{atomic::AtomicI32, Arc}, time::SystemTime};

use kong_rs_protos::{rpc_call::Call, rpc_return::Return, InstanceStatus, PluginInfo, PluginNames, RpcCall, RpcReturn};
use tokio::{net::UnixListener, sync::RwLock};

use crate::{pdk::Pdk, plugin::{ErasedPlugin, ErasedPluginFactory, Phase}, stream::Stream};

// TODO: At the moment, each plugin server can only host a single plugin (Kong limitation.)

struct Instance {
  id: i32,
  start_time: SystemTime,
  plugin: Box<dyn ErasedPlugin + Send + Sync>
}

struct RegisteredFactory {
  time: SystemTime,
  factory: Box<dyn ErasedPluginFactory>
}

#[derive(Clone, serde::Serialize)]
struct Schema {
  name: String,
  fields: serde_json::Value
}

#[derive(Clone, serde::Serialize)]
#[allow(non_snake_case)]
struct ServerInfo {
  Name: String,
  Priority: i32,
  Version: String,
  Schema: Schema,
  Phases: Vec<String>
}

pub struct PluginServerBroker {
  plugin_factories: Arc<RwLock<HashMap<String, RegisteredFactory>>>,
}

impl PluginServerBroker {
  pub fn new() -> Self {
    Self {
      plugin_factories: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub async fn register<F: ErasedPluginFactory + 'static>(&self, factory: F) {
    self.plugin_factories.write().await.insert(factory.get_info().name, RegisteredFactory { time: SystemTime::now(), factory: Box::new(factory) });
  }

  pub async fn run<'a, I: Iterator<Item = String>>(&self, mut args: I) -> anyhow::Result<()> {
    let name = args.next().ok_or(anyhow::anyhow!("Missing name argument"))?;
    let basename = Path::new(&name).file_name().unwrap().to_str().unwrap();

    if args.any(|x| x == "-dump") {
      // Dump
      let factory = self.plugin_factories.read().await;
      if factory.len() != 1 {
        anyhow::bail!("Currently, kong only supports a single plugin per process.");
      }

      let factory = factory.values().next().unwrap();
      let info = factory.factory.get_info();

      let infos = format!("{{\"Protocol\":\"ProtoBuf:1\",\"Plugins\":[{}]}}", serde_json::to_string(&ServerInfo {
        Name: basename.to_owned(),
        Priority: info.priority as i32,
        Version: info.version,
        Schema: Schema { name: info.name, fields: info.schema },
        Phases: info.phases.into_iter().map(|x| Into::<&str>::into(x).to_owned()).collect()
      })?);

      println!("{}", infos);

      return Ok(())
    }

    let socket_addr = format!("/usr/local/kong/{}.socket", basename);
    std::fs::remove_file(&socket_addr).ok();   // Remove if exists, otherwise no-op

    let listener = UnixListener::bind(&socket_addr)?;

    let server = PluginServer::new(self.plugin_factories.clone());
    loop {
      let (stream, _addr) = listener.accept().await?;
      let server = server.clone();
      tokio::spawn(async move { server.handle(Stream::new(stream)).await.unwrap() });
    }
  }
}

#[derive(Clone)]
pub struct PluginServer {
  plugin_factories: Arc<RwLock<HashMap<String, RegisteredFactory>>>,
  instances: Arc<RwLock<HashMap<i32, Instance>>>,
  instance_counter: Arc<AtomicI32>
}

impl PluginServer {
  fn new(plugin_factories: Arc<RwLock<HashMap<String, RegisteredFactory>>>) -> PluginServer {
    Self {
      plugin_factories,
      instances: Arc::new(RwLock::new(HashMap::new())),
      instance_counter: Arc::new(AtomicI32::new(0))
    }
  }

  pub async fn register<F: ErasedPluginFactory + 'static>(&mut self, factory: F) {
    self.plugin_factories.write().await.insert(factory.get_info().name, RegisteredFactory { time: SystemTime::now(), factory: Box::new(factory) });
  }

  pub async fn handle(&self, stream: Stream) -> anyhow::Result<()> {
    loop {
      let req = stream.read_message::<RpcCall>().await?;
      
      if let Some(response) = self.handle_call(stream.clone(), req).await? {
        stream.write_message(&response).await?;
      } else {
        stream.write_frame(&[]).await?;
      }
    }
  }
}

impl PluginServer {
  async fn handle_call(&self, stream: Stream, request: RpcCall) -> anyhow::Result<Option<RpcReturn>> {
    let resp = match request.call {
      Some(Call::CmdGetPluginNames(_)) => {
        Some(Return::PluginNames(PluginNames {
          names: self.plugin_factories.read().await.iter().map(|x| x.0.clone()).collect()
        }))
      },
      Some(Call::CmdGetPluginInfo(get_info)) => {
        let factories = self.plugin_factories.read().await;
        let factory = factories.get(&get_info.name);
        if let Some(factory) = factory {
          let info = factory.factory.get_info();
          let schema = Schema { name: info.name.clone(), fields: info.schema };
          Some(Return::PluginInfo(PluginInfo {
            name: info.name,
            updated_at: factory.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
            loaded_at: factory.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
            phases: info.phases.into_iter().map(|x| Into::<&str>::into(x).to_owned()).collect(),
            version: info.version,
            priority: info.priority,
            schema: serde_json::to_string(&schema)?,
          }))
        } else {
          None
        }
      },
      Some(Call::CmdStartInstance(inst_req)) => {
        let factories = self.plugin_factories.read().await;
        // let factory = factories.get(&inst_req.name);

        // TODO: We can only have one plugin per pluginserver, and it inherits the name of the process.
        //       In such case, the first factory is the only one...
        // TODO: When Kong starts to support multiple plugins, deal with it then.
        let factory = factories.values().next();
        if let Some(factory) = factory {
          let plugin = factory.factory.new(std::str::from_utf8(&inst_req.config)?).await;
          let inst = Instance {
            id: self.instance_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            start_time: SystemTime::now(),
            plugin
          };

          let ret = Return::InstanceStatus(InstanceStatus {
            name: inst_req.name,
            instance_id: inst.id,
            config: None,     // TODO: this isn't currently used in Kong as far as I can tell
            started_at: inst.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
          });

          self.instances.write().await.insert(inst.id, inst);

          Some(ret)
        } else {
          None
        }
      },
      Some(Call::CmdGetInstanceStatus(status_req)) => {
        let instances = self.instances.read().await;
        let inst = instances.get(&status_req.instance_id);
        inst.map(|inst| {
          Return::InstanceStatus(InstanceStatus {
            name: inst.plugin.name(),
            instance_id: inst.id,
            config: None,     // TODO: this isn't currently used in Kong as far as I can tell
            started_at: inst.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
          })
        })
      },
      Some(Call::CmdCloseInstance(close_req)) => {
        self.instances.write().await.remove(&close_req.instance_id);
        None
      },
      Some(Call::CmdHandleEvent(event)) => {
        let phase = Phase::try_from(event.event_name.as_str()).map_err(|_| anyhow::anyhow!("Could not decode phase"))?;
        let instances = self.instances.read().await;
        let inst = instances.get(&event.instance_id);

        if let Some(inst) = inst {
          inst.plugin._call_phase(&phase, &Pdk::new(stream.clone())).await;

          Some(Return::InstanceStatus(InstanceStatus {
            name: inst.plugin.name(),
            instance_id: inst.id,
            config: None,     // TODO: this isn't currently used in Kong as far as I can tell
            started_at: inst.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
          }))
        } else {
          None
        }
      },
      None => None
    };

    Ok(resp.map(|data| RpcReturn { sequence: request.sequence, r#return: Some(data) }))
  }
}
