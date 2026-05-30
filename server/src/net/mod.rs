mod world_runner;

use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};

use color_eyre::eyre::{Result, bail};
use protocol::{
    PlayerId, ServerMessage, WorldMessage,
    net::{ClientPacket, ServerPacket},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
};
use tracing::info;

use crate::net::world_runner::run_world;

/// Owns the networking layer of the game server.
///
/// A [`Server`] is responsible for:
/// - Accepting and managing client connections.
/// - Receiving and decoding client packets.
/// - Sending authoritative server packets to clients.
/// - Creating, tracking, and disconnecting players states with the [`World`].
pub struct Server {
    /// Address the server will bind to when started.
    address: SocketAddr,
    /// A running counter for the current minumum unassigned [`PlayerId`].
    player_id_counter: u32,
}

impl Server {
    /// Constructs a [`Server`] without binding or accepting connections.
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            player_id_counter: 0,
        }
    }

    /// Begins listening for incoming connections on the configured address.
    ///
    /// This function blocks the current thread until the server shuts down
    /// or an unrecoverable error occurs.
    pub async fn run(self) -> Result<()> {
        // sends messages from the server to the world
        let (server_tx, server_rx) = mpsc::channel(1024);
        // sends messages from the world to the server
        let (world_tx, world_rx) = mpsc::channel(1024);

        // spawn a dedicated hardware thread to manage the world
        thread::spawn(move || {
            run_world(Duration::from_millis(50), world_tx, server_rx);
        });

        self.accept_connections(server_tx, world_rx).await
    }

    /// Continually accepts new connection requests, forever blocking the thread.
    async fn accept_connections(
        mut self,
        server_tx: Sender<ServerMessage>,
        mut world_rx: Receiver<WorldMessage>,
    ) -> Result<()> {
        info!("server listening on: {}", self.address);

        let listener = TcpListener::bind(self.address).await?;
        let mut connections = HashMap::new();

        loop {
            tokio::select! {
                // New client connection
                Ok((stream, addr)) = listener.accept() => {
                    let id = PlayerId(self.player_id_counter);
                    self.player_id_counter += 1;

                    let (world_tx, world_rx) = mpsc::channel(32);
                    connections.insert(id, world_tx);

                    info!("incoming connection request from: {addr:?} ({id:?})");

                    let server_tx = server_tx.clone();

                    tokio::spawn(async move {
                        if let Err(reason) = Self::handle_connection(stream, id, server_tx, world_rx).await {
                            info!("client at {addr} disconnected: {reason}");
                        }
                    });
                }

                // Message from the world
                Some(msg) = world_rx.recv() => {
                    match msg {
                        WorldMessage::SendPacket { id, packet } => {
                            if let Some(tx) = connections.get(&id) {
                                let _ = tx.send(packet).await;
                            }
                        }

                        WorldMessage::Broadcast { ids, packet } => {
                            for id in ids {
                                if let Some(tx) = connections.get(&id) {
                                    let _ = tx.send(packet.clone()).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Handles an incoming networking request, managing the entire client lifetime.
    async fn handle_connection(
        stream: TcpStream,
        id: PlayerId,
        server_tx: Sender<ServerMessage>,
        mut world_rx: Receiver<ServerPacket>,
    ) -> Result<()> {
        let (mut rd, mut wr) = stream.into_split();

        let ClientPacket::JoinRequest { username } = Self::read_packet(&mut rd).await? else {
            Self::write_packet(
                &mut wr,
                ServerPacket::Kick {
                    reason: Some("connection request must begin with a join request".into()),
                },
            )
            .await?;

            bail!("invalid connection request");
        };

        let _ = server_tx
            .send(ServerMessage::NewConnection { id, username })
            .await;

        loop {
            tokio::select! {
                packet_result = Self::read_packet(&mut rd) => {
                    let Ok(packet) = packet_result else {
                        break;
                    };
                }

                Some(packet) = world_rx.recv() => {
                    Self::write_packet(&mut wr, packet).await?;
                }
            }
        }

        let _ = server_tx.send(ServerMessage::Disconnect { id }).await;

        Ok(())
    }

    /// Reads a [`ClientPacket`] from a network stream.
    async fn read_packet<R: AsyncRead + Unpin>(rd: &mut R) -> Result<ClientPacket> {
        let len = rd.read_u32().await? as usize;
        let mut buf = vec![0u8; len];
        rd.read_exact(&mut buf).await?;

        Ok(postcard::from_bytes(&buf)?)
    }

    /// Writes a [`ServerPacket`] to a network stream.
    async fn write_packet<W: AsyncWrite + Unpin>(wr: &mut W, packet: ServerPacket) -> Result<()> {
        let buf = postcard::to_allocvec(&packet).unwrap();
        wr.write_u32(buf.len() as u32).await?;
        wr.write_all(&buf).await?;
        Ok(())
    }
}
