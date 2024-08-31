use bevy::{
    app::Plugins,
    core_pipeline::CorePipelinePlugin,
    ecs::system::SystemParam,
    gizmos::GizmoPlugin,
    gltf::GltfPlugin,
    log::{Level, LogPlugin},
    pbr::PbrPlugin,
    prelude::*,
    render::RenderPlugin,
    scene::ScenePlugin,
    sprite::SpritePlugin,
    state::app::StatesPlugin,
};
use lightyear::{client::config::ClientConfig, server::plugin::ServerPlugins};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};
use zusammen_plugin::ZusammenPlugin;

use crate::{
    apps::{make_server_config, NetParameters},
    server::ServerPlugin,
};

use super::{
    apps::{make_client_config, ClientParams, ClientTransportParams, ServerParams, SharedParams},
    client::ClientPlugin,
    shared::SharedPlugin,
};

#[derive(Clone)]
pub struct ServerAppMessage {
    server_params: ServerParams,
    shared_params: SharedParams,
    zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

#[derive(Resource)]
pub struct ClientZusammenAppStuff {
    tx: Option<Sender<ServerAppMessage>>,
    config: ZusammenAppConfig,
}

fn client_app<M, N>(
    client_config: lightyear::client::config::ClientConfig,
    server_tx: ClientZusammenAppStuff,
    client_plugins: impl Plugins<M>,
    shared_plugins: impl Plugins<N>,
) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().set(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        ..default()
    }));

    app.insert_resource(server_tx);

    // Lightyear client plugins
    app.add_plugins(lightyear::prelude::client::ClientPlugins {
        config: client_config,
    });

    app.add_plugins(shared_plugins).add_plugins(client_plugins);

    app
}

fn server_app<M, N>(
    server_config: lightyear::server::config::ServerConfig,
    server_plugins: impl Plugins<M>,
    shared_plugins: impl Plugins<N>,
    should_log: bool,
) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(HierarchyPlugin);
    app.add_plugins(RenderPlugin::default());
    app.add_plugins(ImagePlugin::default());
    app.add_plugins(CorePipelinePlugin::default());
    app.add_plugins(WindowPlugin::default());
    app.add_plugins(ScenePlugin::default());
    app.add_plugins(PbrPlugin::default());
    app.add_plugins(GltfPlugin::default());
    app.add_plugins(TransformPlugin::default());
    app.add_plugins(SpritePlugin::default());
    app.add_plugins(GizmoPlugin);

    if should_log {
        app.add_plugins(LogPlugin {
            level: Level::INFO,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
            ..default()
        });
    }

    app.add_plugins(ServerPlugins {
        config: server_config,
    });

    app.add_plugins(server_plugins).add_plugins(shared_plugins);

    app
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZusammenAppMode {
    Server { port: u16 },
    Client { port: u16, ip: Ipv4Addr },
    Host { port: u16 },
    Lobby,
}

#[derive(Clone)]
pub struct ZusammenAppConfig {
    pub plugin: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
    pub mode: ZusammenAppMode,
}

pub fn run_multiplayer_app(config: ZusammenAppConfig) {
    match config.mode {
        ZusammenAppMode::Server { port } => {
            let shared_params = SharedParams {
                tick_duration: Duration::from_secs_f64(1. / 64.),
            };
            let server_params = ServerParams {
                port,
                local_channel: None,
            };

            let server_config = make_server_config(&server_params, &shared_params);
            let mut app = server_app(
                server_config,
                ServerPlugin {
                    zusammen: config.plugin.clone(),
                },
                SharedPlugin {
                    zusammen: config.plugin.clone(),
                },
                true,
            );
            app.run();
        }

        ZusammenAppMode::Client { port, ip } => {
            let client_params = ClientParams {
                transport: ClientTransportParams::UdpSocket {
                    server_addr: SocketAddr::new(ip.into(), port),
                },
            };
            let shared_params = SharedParams {
                tick_duration: Duration::from_secs_f64(1. / 64.),
            };

            let client_config = make_client_config(&client_params, &shared_params);
            let mut app = client_app(
                client_config,
                ClientZusammenAppStuff {
                    tx: None,
                    config: config.clone(),
                },
                ClientPlugin {
                    zusammen: config.plugin.clone(),
                },
                SharedPlugin {
                    zusammen: config.plugin.clone(),
                },
            );
            app.run();
        }

        ZusammenAppMode::Host { port } => {
            let (tx, rx) = channel::<ServerAppMessage>();

            thread::spawn(move || server_thread(rx));

            let (from_server_send, from_server_recv) = crossbeam_channel::unbounded();
            let (to_server_send, to_server_recv) = crossbeam_channel::unbounded();

            let client_params = ClientParams {
                transport: ClientTransportParams::LocalChannel {
                    recv: from_server_recv,
                    send: to_server_send,
                },
            };
            let shared_params = SharedParams {
                tick_duration: Duration::from_secs_f64(1. / 64.),
            };
            let server_params = ServerParams {
                port,
                local_channel: Some((to_server_recv, from_server_send)),
            };

            let client_config = make_client_config(&client_params, &shared_params);
            let mut app = client_app(
                client_config,
                ClientZusammenAppStuff {
                    tx: None,
                    config: config.clone(),
                },
                ClientPlugin {
                    zusammen: config.plugin.clone(),
                },
                SharedPlugin {
                    zusammen: config.plugin.clone(),
                },
            );
            let server_msg = ServerAppMessage {
                server_params: server_params.clone(),
                shared_params: shared_params.clone(),
                zusammen: config.plugin.clone(),
            };
            tx.send(server_msg).unwrap();
            app.run();
        }

        ZusammenAppMode::Lobby => {
            let (tx, rx) = channel::<ServerAppMessage>();

            thread::spawn(move || server_thread(rx));

            let client_params = ClientParams {
                transport: ClientTransportParams::None,
            };
            let shared_params = SharedParams {
                tick_duration: Duration::from_secs_f64(1. / 64.),
            };

            let client_config = make_client_config(&client_params, &shared_params);
            let mut app = client_app(
                client_config,
                ClientZusammenAppStuff {
                    tx: Some(tx),
                    config: config.clone(),
                },
                ClientPlugin {
                    zusammen: config.plugin.clone(),
                },
                SharedPlugin {
                    zusammen: config.plugin.clone(),
                },
            );
            app.run();
        }
    }
}

fn server_thread(rx: Receiver<ServerAppMessage>) {
    let msg = rx.recv().expect("Did not expect this to fail");

    println!(
        "I received the server config: {:#?}\n{:#?}",
        msg.server_params, msg.shared_params
    );

    let server_config = make_server_config(&msg.server_params, &msg.shared_params);

    server_app(
        server_config,
        ServerPlugin {
            zusammen: msg.zusammen.clone(),
        },
        SharedPlugin {
            zusammen: msg.zusammen,
        },
        false,
    )
    .run();
}

#[derive(SystemParam)]
pub struct ClientZusammenAppManager<'w> {
    server_tx: ResMut<'w, ClientZusammenAppStuff>,
    client_config: ResMut<'w, ClientConfig>,
}

impl ClientZusammenAppManager<'_> {
    pub fn ready_to_go_bros(&mut self, params: NetParameters) {
        let new_client_config = make_client_config(&params.client, &params.shared);
        *self.client_config = new_client_config;

        if let Some(server_params) = params.server.as_ref() {
            let server_msg = ServerAppMessage {
                server_params: server_params.clone(),
                shared_params: params.shared.clone(),
                zusammen: self.server_tx.config.plugin.clone(),
            };
            self.server_tx
                .tx
                .as_mut()
                .unwrap()
                .send(server_msg)
                .unwrap();
        }
    }
}
