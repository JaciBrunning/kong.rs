use std::{collections::HashMap, sync::{atomic::AtomicI32, Arc}, time::SystemTime};

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
struct ServerInfo {
  name: String,
  priority: i32,
  version: String,
  schema: String,
  phases: Vec<String>
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

  pub async fn run<'a, I: Iterator<Item = &'a str>>(&self, mut args: I) -> anyhow::Result<()> {
    let name = args.next().ok_or(anyhow::anyhow!("Missing name argument"))?;

    if args.any(|x| x == "-dump") {
      // Dump
      let factory = self.plugin_factories.read().await;
      if factory.len() != 1 {
        anyhow::bail!("Currently, kong only supports a single plugin per process.");
      }

      let factory = factory.values().next().unwrap();
      let info = factory.factory.get_info();

      println!("{}", serde_json::to_string(&ServerInfo {
        name: info.name,
        priority: info.priority as i32,
        version: info.version,
        schema: info.schema,
        phases: info.phases.into_iter().map(|x| Into::<&str>::into(x).to_owned()).collect()
      })?);

      return Ok(())
    }

    let socket_addr = format!("/usr/local/kong/{}.socket", name);
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
        factory.map(|factory| {
          let info = factory.factory.get_info();
          Return::PluginInfo(PluginInfo {
            name: info.name,
            updated_at: factory.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
            loaded_at: factory.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
            phases: info.phases.into_iter().map(|x| Into::<&str>::into(x).to_owned()).collect(),
            version: info.version,
            priority: info.priority,
            schema: info.schema,
          })
        })
      },
      Some(Call::CmdStartInstance(inst_req)) => {
        let factories = self.plugin_factories.read().await;
        let factory = factories.get(&inst_req.name);
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
          None
        } else {
          None
        }
      },
      None => None
    };

    Ok(resp.map(|data| RpcReturn { sequence: request.sequence, r#return: Some(data) }))
  }
}
