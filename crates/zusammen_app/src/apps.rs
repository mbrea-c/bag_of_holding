//! Utilities for building the Bevy app
//!
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use lightyear::connection::client::NetConfig;
use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
use lightyear::prelude::client::{ClientConfig, ClientTransport};
use lightyear::prelude::*;
use lightyear::prelude::{client, server};
use lightyear::server::config::ServerConfig;
use lightyear::transport::LOCAL_SOCKET;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

const PROTOCOL_ID: u64 = 0;
const PRIVATE_KEY: [u8; PRIVATE_KEY_BYTES] = [
    12, 58, 98, 88, 72, 254, 12, 121, 99, 83, 211, 132, 199, 12, 58, 98, 88, 72, 254, 12, 121, 99,
    83, 211, 132, 199, 241, 1, 0, 54, 71, 13,
];
const REPLICATION_SEND_INTERVAL: Duration = Duration::from_millis(40);

fn shared_config(params: &SharedParams) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: REPLICATION_SEND_INTERVAL,
        tick: TickConfig {
            tick_duration: params.tick_duration,
        },
        mode: Mode::Separate,
    }
}

#[derive(Resource, Clone, Debug)]
pub struct NetParameters {
    pub client: ClientParams,
    pub server: Option<ServerParams>,
    pub shared: SharedParams,
}

#[derive(Clone, Debug)]
pub struct SharedParams {
    pub tick_duration: Duration,
}

#[derive(Clone, Debug)]
pub struct ClientParams {
    pub transport: ClientTransportParams,
}

#[derive(Clone, Debug)]
pub struct ServerParams {
    pub port: u16,
    pub local_channel: Option<(Receiver<Vec<u8>>, Sender<Vec<u8>>)>,
}

#[derive(Clone, Debug)]
pub enum ClientTransportParams {
    UdpSocket {
        server_addr: SocketAddr,
    },
    LocalChannel {
        recv: Receiver<Vec<u8>>,
        send: Sender<Vec<u8>>,
    },
    /// e.g. while in main menu, configuring connection
    None,
}

pub fn make_client_config(
    client_params: &ClientParams,
    shared_params: &SharedParams,
) -> ClientConfig {
    let client_id = rand::random();
    let (transport, server_addr) = match client_params.transport.clone() {
        ClientTransportParams::UdpSocket { server_addr } => {
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);

            (ClientTransport::UdpSocket(client_addr), server_addr)
        }
        ClientTransportParams::LocalChannel { recv, send } => {
            (ClientTransport::LocalChannel { recv, send }, LOCAL_SOCKET)
        }
        // This one does not matter too much, we won't try to connect until settings are updated
        ClientTransportParams::None => (ClientTransport::Dummy, LOCAL_SOCKET),
    };

    let io_config = client::IoConfig::from_transport(transport);

    let auth = client::Authentication::Manual {
        server_addr,
        client_id,
        private_key: PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    let net_config = NetConfig::Netcode {
        auth,
        io: io_config,
        config: client::NetcodeConfig::default(),
    };

    let client_config = ClientConfig {
        shared: shared_config(shared_params),
        net: net_config,
        ..default()
    };

    client_config
}

pub fn make_server_config(
    server_params: &ServerParams,
    shared_params: &SharedParams,
) -> ServerConfig {
    let mut extra_transport_configs = vec![];
    if let Some((recv, send)) = server_params.local_channel.clone() {
        extra_transport_configs.push(server::ServerTransport::Channels {
            channels: vec![(LOCAL_SOCKET, recv, send)],
        });
    }

    // configure the network configuration
    let mut net_configs = vec![build_server_netcode_config(
        server::ServerTransport::UdpSocket(SocketAddr::new(
            Ipv4Addr::UNSPECIFIED.into(),
            server_params.port,
        )),
    )];

    let extra_net_configs = extra_transport_configs
        .into_iter()
        .map(|c| build_server_netcode_config(c));
    net_configs.extend(extra_net_configs);

    let server_config = ServerConfig {
        shared: shared_config(shared_params),
        net: net_configs,
        replication: ReplicationConfig {
            send_interval: REPLICATION_SEND_INTERVAL,
            ..default()
        },
        ..default()
    };

    server_config
}

fn build_server_netcode_config(transport_config: server::ServerTransport) -> server::NetConfig {
    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(PROTOCOL_ID)
        .with_key(PRIVATE_KEY);

    let io_config = server::IoConfig {
        transport: transport_config,
        ..default()
    };

    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
    }
}
