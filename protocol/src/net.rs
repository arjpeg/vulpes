use serde::{Deserialize, Serialize};

use crate::{PlayerId, PlayerInfo};

/// A packet sent from the client to the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ClientPacket {
    /// The client wants to connect to the server.
    ///
    /// Must be the first packet sent from the client to the server to establish a valid
    /// connection. If accepted, the server will respond with a [ServerPacket::Welcome]
    JoinRequest {
        /// The player's requested in game username.
        username: String,
    },
}

/// A packet sent from the server to the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerPacket {
    /// The server rejected and disconnected a connection.
    Kick {
        /// A human readable reason for why the server aborted the connection.
        reason: Option<String>,
    },

    /// Response to a successful [ClientPacket::JoinRequest].
    ///
    /// This is the synchronization point; after receiving this, the client
    /// should consider the world simulation "started."
    Welcome {
        /// The identity assigned to the local client.
        local_player: PlayerInfo,
        /// An initial snapshot of all players currently present in the world.
        existing_players: Vec<PlayerInfo>,
    },

    /// Notifies clients that a new player has joined the server.
    ///
    /// Unlike [ServerPacket::Welcome], this packet is broadcast to existing clients to inform
    /// them of another player joining.
    PlayerJoined(PlayerInfo),

    /// Notifies clients that a player has left the server.
    PlayerLeft(PlayerId),
}
