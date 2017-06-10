use std::fmt::Debug;
use std::io::Cursor;

use byteorder::{BigEndian, WriteBytesExt};

use utils::slice_ext::SliceExt;
use super::{Digest, DigestAlgorithm};


pub trait Chunk: AsMut<[u8]> + Debug + Clone {
    fn new() -> Self;
    fn len() -> usize;
}

pub trait ChunkedDigestAlgorithm {
    type Chunk: Chunk;
    type Digest: Digest;

    fn new() -> Self;
    fn update_chunk(&mut self, chunk: &[u8]);
    fn digest(self) -> Self::Digest;
}

#[derive(Debug, Clone)]
pub struct ChunkedDigestWrapper<Inner: ChunkedDigestAlgorithm> {
    // Hash state
    inner: Inner,
    // Message length
    ml: u64,
    // In-progress chunk
    buffer_len: usize,
    buffer: Inner::Chunk
}

impl<Inner: ChunkedDigestAlgorithm> DigestAlgorithm for ChunkedDigestWrapper<Inner> {
    type Digest = Inner::Digest;

    fn new() -> Self {
        ChunkedDigestWrapper {
            inner: Inner::new(),
            ml: 0,
            buffer_len: 0,
            buffer: Inner::Chunk::new()
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        // Update message length (in bits)
        self.ml += (input.len() as u64)*8;

        let buffer_len = self.buffer_len;
        let chunk_len = Inner::Chunk::len();
        let mut buffer = self.buffer.as_mut();

        // If buffer is already partially filled
        if buffer_len > 0 {
            let remaining = chunk_len - buffer_len;
            // If input is not long enough to fill remaining space in buffer
            if input.len() < remaining {
                // Copy the whole input into the buffer and return
                buffer[buffer_len..buffer_len + input.len()].copy_from_slice(&input[..]);
                self.buffer_len += input.len();
                return;
            } else {
                // Fill the remaining space with input data and process the chunk
                buffer[buffer_len..].copy_from_slice(&input[0..remaining]);
                input = &input[remaining..];
                self.inner.update_chunk(buffer);
            }
        }

        // Buffer is empty at this point, so can directly read chunks from input
        while input.len() >= chunk_len {
            self.inner.update_chunk(&input[0..chunk_len]);
            input = &input[chunk_len..];
        }

        // Partially fill the buffer with any left-over input
        buffer[0..input.len()].copy_from_slice(input);
        self.buffer_len = input.len();
    }

    fn digest(mut self) -> Self::Digest {
        let chunk_len = Inner::Chunk::len();
        let mut buffer = self.buffer.as_mut();

        // Append 1 bit
        buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        // Ensure there's room to write the message length
        if self.buffer_len + 8 > chunk_len {
            buffer[self.buffer_len..].fill_copy(0);
            self.buffer_len = 0;
            self.inner.update_chunk(buffer);
        }

        // Fill remaining space with zeros followed by the message length
        buffer[self.buffer_len..chunk_len-8].fill_copy(0);
        Cursor::new(&mut buffer[chunk_len-8..]).write_u64::<BigEndian>(self.ml).unwrap();
        // Final chunk computation
        self.inner.update_chunk(buffer);
        self.inner.digest()
    }
}
