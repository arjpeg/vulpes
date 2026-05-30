pub mod net;

use serde::{Deserialize, Serialize};

use crate::net::ServerPacket;

/// A unique identifier for a connected player, managed by the [Server].
///
/// # Guarantees
/// - IDs are unique within a single execution session.
/// - IDs are never reused even after a player disconnects to prevent "ghost" messaging.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(pub u32);

/// A data-transfer object representing a player's public-facing identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerInfo {
    /// The unique identifier used for identifying this player across the network.
    pub id: PlayerId,
    /// The display name chosen by this player.
    pub username: String,

    /// The server managed world space position of this player.
    pub position: [f32; 3],
}

/// A unique identifier for a type of block managed by a block registry at runtime.
///
/// A [BlockId] has no intrinsic meaning on its own; its interpretation depends
/// entirely on the registry negotiated during connection.
///
/// # Guarantees
/// - IDs are stable for the lifetime of a single server session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u16);

/// A cubic region of space that covers 32^3 blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// The world space position of the bottom left block of this chunk.
    pub position: [f32; 3],
    /// The encoded data stored in this chunk.
    pub encoded: ChunkData,
}

/// The length, in blocks, of a [Chunk] across each axis.
pub const CHUNK_LENGTH: usize = 16;

/// The total volume of blocks contained within a [Chunk].
pub const CHUNK_VOLUME: usize = CHUNK_LENGTH.pow(3);

/// All blocks stored in a [Chunk] under some encoding scheme.
///
/// All encoding schemes store the blocks in the following order: starting from (0, 0, 0)
/// in chunk space, move along the +X axis, then +Z, then finally +Y.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkData {
    /// The block data is not compressed at all.
    Raw(Vec<BlockId>),
}

/// Messages that a server can send to the world simulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerMessage {
    /// A new player joined the world.
    NewConnection {
        /// The player's assigned id.
        id: PlayerId,
        /// The player's selected in game username.
        username: String,
    },

    /// A player left the world.
    Disconnect {
        /// The player's assigned id.
        ///
        /// Will not be reused by any future players.
        id: PlayerId,
    },
}

/// Messages that a world simulation can send to the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldMessage {
    /// The server should send a packet to the given player.
    SendPacket {
        /// The recipient client's id.
        id: PlayerId,
        /// The packet to be sent.
        packet: ServerPacket,
    },

    /// The server should broadcast the packet to the players specified.
    Broadcast {
        /// The ids of the players to recieve the packet.
        ids: Vec<PlayerId>,
        /// The packet to be sent.
        packet: ServerPacket,
    },
}
