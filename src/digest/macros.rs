
macro_rules! define_digest {
    ($digest:ident, $size:expr) => {
        #[derive(Copy)]
        pub struct $digest([u8; $size]);

        impl AsRef<[u8]> for $digest {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl Into<Box<[u8]>> for $digest {
            fn into(self) -> Box<[u8]> {
                Box::new(self.0)
            }
        }

        impl ::std::fmt::Debug for $digest {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}(\"", stringify!($digest))?;
                for byte in &self.0[..] {
                    write!(f, "{:02x}", byte)?;
                }
                write!(f, "\")")
            }
        }

        impl Clone for $digest {
            fn clone(&self) -> Self {
                $digest(self.0)
            }
        }

        impl $crate::digest::Digest for $digest {}
    }
}

macro_rules! define_chunk {
    ($chunk:ident, $size:expr) => {
        #[derive(Copy)]
        pub struct $chunk([u8; $size]);

        impl AsMut<[u8]> for $chunk {
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }

        impl ::std::fmt::Debug for $chunk {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                for byte in &self.0[..] {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
        }

        impl Clone for $chunk {
            fn clone(&self) -> Self {
                $chunk(self.0)
            }
        }

        impl $crate::digest::chunked::Chunk for $chunk {
            fn new() -> Self {
                $chunk([0; $size])
            }
            fn len() -> usize {
                $size
            }
        }
    }
}
