use protocol::{BlockId, CHUNK_LENGTH};

/// A cubic region of blocks.
#[derive(Debug)]
pub struct Chunk {
    blocks: [[[BlockId; CHUNK_LENGTH]; CHUNK_LENGTH]; CHUNK_LENGTH],
}
