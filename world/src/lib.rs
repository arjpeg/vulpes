mod chunk;

use std::{collections::HashMap, mem, time::Duration};

use protocol::{PlayerId, PlayerInfo, ServerMessage, WorldMessage, net::ServerPacket};

use rand::Rng;

/// The radius, in chunks, around each player for chunks are loaded.
const CHUNK_LOAD_RADIUS: usize = 8;

/// Authoritative representation of the game world.
///
/// The [`World`] owns all voxel data and world-level state. It is responsible for
/// loading, generating, and unloading chunks based on player presence, and
/// for applying deterministic updates to chunk contents.
pub struct World {
    /// A map of all loaded [`Chunks`] surrounding players.
    chunks: Vec<()>,

    /// A list of all connected players, and their corresponding data.
    players: HashMap<PlayerId, Player>,

    /// A list of the outbound messages to pass back to the server.
    outbound: Vec<WorldMessage>,

    /// The minimum duration between update cycles.
    tick_rate: Duration,
}

/// The server managed world data associated with each connected player.
#[derive(Debug, Clone)]
pub struct Player {
    /// The unique, server-assigned id.
    id: PlayerId,
    /// The player's in game username.
    username: String,

    /// The player's server-managed world space position.
    position: [f32; 3],
}

impl World {
    /// Creates a new, empty [`World`].
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            chunks: Vec::new(),
            players: HashMap::new(),
            outbound: Vec::new(),
            tick_rate,
        }
    }

    /// Handles an incoming [`ServerMessage`].
    pub fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::NewConnection { id, username } => {
                let mut rng = rand::rng();

                let position = [
                    rng.random_range(-5.0..=5.0),
                    rng.random_range(-5.0..=5.0),
                    rng.random_range(-5.0..=5.0),
                ];

                let player = Player {
                    id,
                    username,
                    position,
                };

                let player_info = PlayerInfo::from(player.clone());

                // notify everyone else that this player joined
                self.outbound.push(WorldMessage::Broadcast {
                    ids: self.players.keys().cloned().collect(),
                    packet: ServerPacket::PlayerJoined(player_info.clone()),
                });

                // alert the connecting player of their assigned id and of the other players
                self.outbound.push(WorldMessage::SendPacket {
                    id: player.id,
                    packet: ServerPacket::Welcome {
                        local_player: player_info,
                        existing_players: self
                            .players
                            .values()
                            .cloned()
                            .map(Player::into)
                            .collect(),
                    },
                });

                assert!(self.players.insert(player.id, player).is_none());
            }

            ServerMessage::Disconnect { id } => {
                assert!(self.players.remove(&id).is_some());

                // notify everyone that this player left
                self.outbound.push(WorldMessage::Broadcast {
                    ids: self.players.keys().cloned().collect(),
                    packet: ServerPacket::PlayerLeft(id),
                });
            }
        }
    }

    /// Drains all the outbound [`WorldMessage`]s to be processed on the server.
    pub fn poll_messages(&mut self) -> Vec<WorldMessage> {
        mem::take(&mut self.outbound)
    }

    /// Advances the world by one fixed timestep, i.e. one "tick".
    /// Assumes the `tick_rate` is constant between updates.
    pub fn tick(&mut self) {}

    /// Returns the tick rate of this [`World`].
    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}

impl From<Player> for PlayerInfo {
    fn from(player: Player) -> Self {
        Self {
            id: player.id,
            username: player.username,
            position: player.position,
        }
    }
}
