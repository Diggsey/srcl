use std::io::Cursor;
use super::chunked::{ChunkedDigestAlgorithm, ChunkedDigestWrapper};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};


define_digest!(SHA224Digest, 28);
define_digest!(SHA256Digest, 32);
define_digest!(SHA384Digest, 48);
define_digest!(SHA512Digest, 64);
define_digest!(SHA512IVGenDigest, 64);
define_digest!(SHA512T224Digest, 28);
define_digest!(SHA512T256Digest, 32);

define_chunk!(SHA256Chunk, 64);
define_chunk!(SHA512Chunk, 128);


fn sha256_update_chunk(self_h: &mut [u32; 8], chunk: &[u8]) {
    // Compute 64 words
    let mut w = [0; 64];
    let mut reader = Cursor::new(chunk);
    for i in 0..16 {
        w[i] = reader.read_u32::<BigEndian>().unwrap();
    }
    for i in 16..64 {
        let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
        let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
        w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
    }

    // Round constants
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    ];

    // Copy hash state
    let mut h = *self_h;

    // Bitwise functions
    let choose   = |x: u32, y: u32, z: u32| (x & y) | (!x & z);
    let majority = |x: u32, y: u32, z: u32| (x & y) | (y & z) | (x & z);

    for i in 0..64 {
        let s1 = h[4].rotate_right(6) ^ h[4].rotate_right(11) ^ h[4].rotate_right(25);
        let temp1 = h[7].wrapping_add(s1).wrapping_add(choose(h[4], h[5], h[6])).wrapping_add(K[i]).wrapping_add(w[i]);
        let s0 = h[0].rotate_right(2) ^ h[0].rotate_right(13) ^ h[0].rotate_right(22);
        let temp2 = s0.wrapping_add(majority(h[0], h[1], h[2]));

        for i in (0..7).rev() {
            h[i+1] = h[i];
        }
        h[4] = h[4].wrapping_add(temp1);
        h[0] = temp1.wrapping_add(temp2);
    }

    // Update hash state
    for i in 0..8 {
        self_h[i] = self_h[i].wrapping_add(h[i]);
    }
}

fn sha512_update_chunk(self_h: &mut [u64; 8], chunk: &[u8]) {
    // Compute 80 words
    let mut w = [0; 80];
    let mut reader = Cursor::new(chunk);
    for i in 0..16 {
        w[i] = reader.read_u64::<BigEndian>().unwrap();
    }
    for i in 16..80 {
        let s0 = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
        let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
        w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
    }

    // Round constants
    const K: [u64; 80] = [
        0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, 0x3956c25bf348b538, 
        0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242, 0x12835b0145706fbe, 
        0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 
        0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 
        0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5, 0x983e5152ee66dfab, 
        0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725, 
        0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 
        0x53380d139d95b3df, 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b, 
        0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218, 
        0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8, 0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 
        0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 
        0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec, 
        0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b, 0xca273eceea26619c, 
        0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba, 0x0a637dc5a2c898a6, 
        0x113f9804bef90dae, 0x1b710b35131c471b, 0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 
        0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817
    ];

    // Copy hash state
    let mut h = *self_h;

    // Bitwise functions
    let choose   = |x: u64, y: u64, z: u64| (x & y) | (!x & z);
    let majority = |x: u64, y: u64, z: u64| (x & y) | (y & z) | (x & z);

    for i in 0..80 {
        let s1 = h[4].rotate_right(14) ^ h[4].rotate_right(18) ^ h[4].rotate_right(41);
        let temp1 = h[7].wrapping_add(s1).wrapping_add(choose(h[4], h[5], h[6])).wrapping_add(K[i]).wrapping_add(w[i]);
        let s0 = h[0].rotate_right(28) ^ h[0].rotate_right(34) ^ h[0].rotate_right(39);
        let temp2 = s0.wrapping_add(majority(h[0], h[1], h[2]));

        for i in (0..7).rev() {
            h[i+1] = h[i];
        }
        h[4] = h[4].wrapping_add(temp1);
        h[0] = temp1.wrapping_add(temp2);
    }

    // Update hash state
    for i in 0..8 {
        self_h[i] = self_h[i].wrapping_add(h[i]);
    }
}

#[derive(Debug, Clone)]
pub struct SHA224Chunked {
    h: [u32; 8],
}

impl ChunkedDigestAlgorithm for SHA224Chunked {
    type Digest = SHA224Digest;
    type Chunk = SHA256Chunk;

    fn new() -> Self {
        SHA224Chunked {
            h: [0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha256_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 28];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h[0..7] {
                writer.write_u32::<BigEndian>(h).unwrap();
            }
        }
        SHA224Digest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA256Chunked {
    h: [u32; 8],
}

impl ChunkedDigestAlgorithm for SHA256Chunked {
    type Digest = SHA256Digest;
    type Chunk = SHA256Chunk;

    fn new() -> Self {
        SHA256Chunked {
            h: [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha256_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 32];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h {
                writer.write_u32::<BigEndian>(h).unwrap();
            }
        }
        SHA256Digest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA384Chunked {
    h: [u64; 8],
}

impl ChunkedDigestAlgorithm for SHA384Chunked {
    type Digest = SHA384Digest;
    type Chunk = SHA512Chunk;

    fn new() -> Self {
        SHA384Chunked {
            h: [
                0xcbbb9d5dc1059ed8, 0x629a292a367cd507, 0x9159015a3070dd17, 0x152fecd8f70e5939, 
                0x67332667ffc00b31, 0x8eb44a8768581511, 0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4
            ]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha512_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 48];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h[0..6] {
                writer.write_u64::<BigEndian>(h).unwrap();
            }
        }
        SHA384Digest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA512Chunked {
    h: [u64; 8],
}

impl ChunkedDigestAlgorithm for SHA512Chunked {
    type Digest = SHA512Digest;
    type Chunk = SHA512Chunk;

    fn new() -> Self {
        SHA512Chunked {
            h: [
                0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1, 
                0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179
            ]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha512_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 64];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h {
                writer.write_u64::<BigEndian>(h).unwrap();
            }
        }
        SHA512Digest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA512IVGenChunked {
    h: [u64; 8],
}

impl ChunkedDigestAlgorithm for SHA512IVGenChunked {
    type Digest = SHA512IVGenDigest;
    type Chunk = SHA512Chunk;

    fn new() -> Self {
        let mut h = SHA512Chunked::new().h;
        for elem in &mut h {
            *elem ^= 0xa5a5a5a5a5a5a5a5;
        }
        SHA512IVGenChunked {
            h: h
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha512_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 64];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h {
                writer.write_u64::<BigEndian>(h).unwrap();
            }
        }
        SHA512IVGenDigest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA512T224Chunked {
    h: [u64; 8],
}

impl ChunkedDigestAlgorithm for SHA512T224Chunked {
    type Digest = SHA512T224Digest;
    type Chunk = SHA512Chunk;

    fn new() -> Self {
        SHA512T224Chunked {
            h: [
                0x8c3d37c819544da2, 0x73e1996689dcd4d6, 0x1dfab7ae32ff9c82, 0x679dd514582f9fcf,
                0x0f6d2b697bd44da8, 0x77e36f7304c48942, 0x3f9d85a86a1d36c8, 0x1112e6ad91d692a1
            ]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha512_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 28];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h[0..4] {
                // Last item will write partial results
                let _ = writer.write_u64::<BigEndian>(h);
            }
        }
        SHA512T224Digest(result)
    }
}

#[derive(Debug, Clone)]
pub struct SHA512T256Chunked {
    h: [u64; 8],
}

impl ChunkedDigestAlgorithm for SHA512T256Chunked {
    type Digest = SHA512T256Digest;
    type Chunk = SHA512Chunk;

    fn new() -> Self {
        SHA512T256Chunked {
            h: [
                0x22312194fc2bf72c, 0x9f555fa3c84c64c2, 0x2393b86b6f53b151, 0x963877195940eabd,
                0x96283ee2a88effe3, 0xbe5e1e2553863992, 0x2b0199fc2c85b8aa, 0x0eb72ddc81c52ca2
            ]
        }
    }

    fn update_chunk(&mut self, chunk: &[u8]) {
        sha512_update_chunk(&mut self.h, chunk);
    }

    fn digest(self) -> Self::Digest {
        let mut result = [0; 32];
        {
            let mut writer = Cursor::new(&mut result[..]);
            for &h in &self.h[0..4] {
                writer.write_u64::<BigEndian>(h).unwrap();
            }
        }
        SHA512T256Digest(result)
    }
}

pub type SHA224 = ChunkedDigestWrapper<SHA224Chunked>;
pub type SHA256 = ChunkedDigestWrapper<SHA256Chunked>;
pub type SHA384 = ChunkedDigestWrapper<SHA384Chunked>;
pub type SHA512 = ChunkedDigestWrapper<SHA512Chunked>;
pub type SHA512IVGen = ChunkedDigestWrapper<SHA512IVGenChunked>;
pub type SHA512T224 = ChunkedDigestWrapper<SHA512T224Chunked>;
pub type SHA512T256 = ChunkedDigestWrapper<SHA512T256Chunked>;

#[cfg(test)]
mod tests {
    use super::super::DigestAlgorithm;
    use super::*;

    fn test<A: DigestAlgorithm>(input: &[u8], expected: &[u8]) {
        let actual = A::compute(input);
        println!("A: {:?}", actual.as_ref());
        println!("B: {:?}", expected);
        assert!(actual.as_ref() == expected)
    }

    #[test]
    fn sha224() {
        test::<SHA224>(b"", base16!("D14A028C2A3A2BC9476102BB288234C415A2B01F828EA62AC5B3E42F"));
        test::<SHA224>(b"The quick brown fox jumps over the lazy dog", base16!("730E109BD7A8A32B1CB9D9A09AA2325D2430587DDBC0C38BAD911525"));
    }

    #[test]
    fn sha256() {
        test::<SHA256>(b"", base16!("E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"));
        test::<SHA256>(b"The quick brown fox jumps over the lazy dog", base16!("D7A8FBB307D7809469CA9ABCB0082E4F8D5651E46D3CDB762D02D0BF37C9E592"));
    }

    #[test]
    fn sha384() {
        test::<SHA384>(b"", base16!("38B060A751AC96384CD9327EB1B1E36A21FDB71114BE07434C0CC7BF63F6E1DA274EDEBFE76F65FBD51AD2F14898B95B"));
        test::<SHA384>(b"The quick brown fox jumps over the lazy dog", base16!("CA737F1014A48F4C0B6DD43CB177B0AFD9E5169367544C494011E3317DBF9A509CB1E5DC1E85A941BBEE3D7F2AFBC9B1"));
    }

    #[test]
    fn sha512() {
        test::<SHA512>(b"", base16!("CF83E1357EEFB8BDF1542850D66D8007D620E4050B5715DC83F4A921D36CE9CE47D0D13C5D85F2B0FF8318D2877EEC2F63B931BD47417A81A538327AF927DA3E"));
        test::<SHA512>(b"The quick brown fox jumps over the lazy dog", base16!("07E547D9586F6A73F73FBAC0435ED76951218FB7D0C8D788A309D785436BBB642E93A252A954F23912547D1E8A3B5ED6E1BFD7097821233FA0538F3DB854FEE6"));
    }

    #[test]
    fn sha512_iv_gen() {
        test::<SHA512IVGen>(b"SHA-512/224", base16!("8C3D37C819544DA273E1996689DCD4D61DFAB7AE32FF9C82679DD514582F9FCF0F6D2B697BD44DA877E36F7304C489423F9D85A86A1D36C81112E6AD91D692A1"));
        test::<SHA512IVGen>(b"SHA-512/256", base16!("22312194FC2BF72C9F555FA3C84C64C22393B86B6F53B151963877195940EABD96283EE2A88EFFE3BE5E1E25538639922B0199FC2C85B8AA0EB72DDC81C52CA2"));
    }

    #[test]
    fn sha512_t224() {
        test::<SHA512T224>(b"", base16!("6ED0DD02806FA89E25DE060C19D3AC86CABB87D6A0DDD05C333B84F4"));
        test::<SHA512T224>(b"The quick brown fox jumps over the lazy dog", base16!("944CD2847FB54558D4775DB0485A50003111C8E5DAA63FE722C6AA37"));
    }

    #[test]
    fn sha512_t256() {
        test::<SHA512T256>(b"", base16!("C672B8D1EF56ED28AB87C3622C5114069BDD3AD7B8F9737498D0C01ECEF0967A"));
        test::<SHA512T256>(b"The quick brown fox jumps over the lazy dog", base16!("DD9D67B371519C339ED8DBD25AF90E976A1EEEFD4AD3D889005E532FC5BEF04D"));
    }

}
