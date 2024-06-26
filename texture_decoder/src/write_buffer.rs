use std::slice::ChunksExactMut;

pub struct WriteBuff {
    buffer: Box<[u8]>,
    chunk_size: usize,
}

impl WriteBuff {
    pub(crate) fn new(size: usize, chunk_size: usize) -> Self {
        let buf = vec![0; size];
        Self { buffer: buf.into_boxed_slice(), chunk_size }
    }

    pub(crate) fn inner(self) -> Box<[u8]> {
        self.buffer
    }
}

impl WriteBuff {
    pub fn as_chunks(&mut self) -> ChunksExactMut<'_, u8> {
        self.buffer.chunks_exact_mut(self.chunk_size)
    }
}
