use std::io::Cursor;
use super::chunked::{ChunkedDigestAlgorithm, ChunkedDigestWrapper};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

define_digest!(SHA1Digest, 20);
define_chunk!(SHA1Chunk, 64);

#[derive(Debug, Clone)]
pub struct SHA1Chunked {
    h: [u32; 5],
}

impl ChunkedDigestAlgorithm for SHA1Chunked {
    type Digest = SHA1Digest;
    type Chunk = SHA1Chunk;

    fn new() -> Self {
        SHA1Chunked {
            h: [
                0x67452301,
                0xEFCDAB89,
                0x98BADCFE,
                0x10325476,
                0xC3D2E1F0
            ]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        // Compute 80 words
        let mut w = [0; 80];
        let mut reader = Cursor::new(chunk);
        for i in 0..16 {
            w[i] = reader.read_u32::<BigEndian>().unwrap();
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }

        // Copy hash state
        let mut h = self.h;

        // Bitwise functions
        let choose   = |x: u32, y: u32, z: u32| (x & y) | (!x & z);
        let parity   = |x: u32, y: u32, z: u32| x ^ y ^ z;
        let majority = |x: u32, y: u32, z: u32| (x & y) | (y & z) | (x & z);

        for i in 0..80 {
            let (f, k) = match i {
                 0...19 => (  choose(h[1], h[2], h[3]), 0x5A827999),
                20...39 => (  parity(h[1], h[2], h[3]), 0x6ED9EBA1),
                40...59 => (majority(h[1], h[2], h[3]), 0x8F1BBCDC),
                60...79 => (  parity(h[1], h[2], h[3]), 0xCA62C1D6),
                      _ => unreachable!()
            };
            let temp = h[0].rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(h[4])
                .wrapping_add(k)
                .wrapping_add(w[i]);
            
            for i in (0..4).rev() {
                h[i+1] = h[i];
            }
            h[2] = h[2].rotate_left(30);
            h[0] = temp;
        }

        // Update hash state
        for i in 0..5 {
            self.h[i] = self.h[i].wrapping_add(h[i]);
        }
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 20];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h {
                writer.write_u32::<BigEndian>(h).unwrap();
            }
        }
        SHA1Digest(result)
    }
}

pub type SHA1 = ChunkedDigestWrapper<SHA1Chunked>;

#[cfg(test)]
mod tests {
    use super::super::DigestAlgorithm;
    use super::*;

    fn test_sha1(input: &[u8], expected: &[u8]) {
        let mut state = SHA1::new();
        state.update(input);
        let actual = state.digest();
        println!("A: {:?}", actual);
        println!("B: {:?}", expected);
        assert!(actual.as_ref() == expected)
    }

    #[test]
    fn sha1() {
        test_sha1(b"", base16!("DA39A3EE5E6B4B0D3255BFEF95601890AFD80709"));
        test_sha1(b"@", base16!("9A78211436F6D425EC38F5C4E02270801F3524F8"));
        test_sha1(b"The quick brown fox jumps over the lazy dog", base16!("2FD4E1C67A2D28FCED849EE1BB76E7391B93EB12"));
    }
}
