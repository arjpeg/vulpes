use std::{
    thread,
    time::{Duration, Instant},
};

use protocol::{ServerMessage, WorldMessage};
use tokio::sync::mpsc::{Receiver, Sender};
use world::World;

/// Executes the [`World`].
///
/// The runner enforces real-time scheduling, while the [`World`] itself
/// remains deterministic and unaware of wall-clock timing.
pub fn run_world(
    tick_rate: Duration,
    world_tx: Sender<WorldMessage>,
    mut server_rx: Receiver<ServerMessage>,
) {
    let mut world = World::new(tick_rate);
    let mut last_tick = Instant::now();

    loop {
        while let Ok(message) = server_rx.try_recv() {
            world.handle_message(message);
        }

        // catch up to the next tick
        while last_tick.elapsed() > world.tick_rate() {
            world.tick();
            last_tick += world.tick_rate();
        }

        for message in world.poll_messages() {
            let _ = world_tx.blocking_send(message);
        }

        let next_tick = last_tick + world.tick_rate();
        let now = Instant::now();

        if now < next_tick {
            thread::sleep(next_tick - now);
        }
    }
}
