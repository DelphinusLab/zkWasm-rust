use crate::keccak_finalize;
use crate::keccak_new;
use crate::keccak_push;
pub struct KeccakHasher(u64);

const RATE_BYTES: usize = 136; // 1088 bits
const RATE_U64S: usize = 17;

impl KeccakHasher {
    pub fn new() -> Self {
        unsafe {
            keccak_new(1u64);
        }
        KeccakHasher(0u64)
    }

    #[inline(always)]
    fn absorb_rate_block(bytes: &[u8]) -> Vec<u64> {
        let r = (0..RATE_U64S)
            .map(|i| {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
                u64::from_le_bytes(buf)
            })
            .collect::<Vec<_>>();
        r
    }

    pub fn keccak256(input: &[u8], len: usize) -> [u64; 4] {
        let mut hasher = Self::new();
        // ===============================
        // 1. absorb  136-bytes =17-u64 block
        // ===============================
        let mut offset = 0;
        while offset + RATE_BYTES <= len {
            let datas = Self::absorb_rate_block(&input[offset..]);
            for x in datas {
                hasher.update(x);
            }
            offset += RATE_BYTES;
        }
        // ===============================
        // 2. remainding bytes + pad10*1
        // ===============================
        let rem = len - offset;
        // zeroed rate buffer
        let mut last = [0u8; RATE_BYTES];
        if rem > 0 {
            last[..rem].copy_from_slice(&input[offset..]);
        }
        // ---- pad ----
        last[rem] ^= 0x01;
        last[RATE_BYTES - 1] ^= 0x80;

        // absorb padded block
        let datas = Self::absorb_rate_block(&last[..]);
        hasher.finalize(&datas)
    }

    pub fn update(&mut self, v: u64) {
        unsafe {
            keccak_push(v);
        }
        self.0 += 1;
        if self.0 == 17 {
            unsafe {
                keccak_finalize();
                keccak_finalize();
                keccak_finalize();
                keccak_finalize();
                keccak_new(0u64);
            }
            self.0 = 0;
        }
    }

    pub fn finalize(&mut self, datas: &[u64]) -> [u64; 4] {
        for v in datas {
            unsafe {
                keccak_push(*v);
            }
            self.0 += 1;
        }
        assert_eq!(self.0, RATE_U64S as u64);

        unsafe {
            [
                keccak_finalize(),
                keccak_finalize(),
                keccak_finalize(),
                keccak_finalize(),
            ]
        }
    }
}
