use crate::keccak_new;
use crate::keccak_push;
use crate::keccak_finalize;
pub struct KeccakHasher (u64);
impl KeccakHasher {
    pub fn new() -> Self {
        unsafe {
            keccak_new(1u64);
        }
        KeccakHasher(0u64)
    }

    pub fn hash(data: &[u64]) -> [u64; 4] {
        let mut hasher = Self::new();
        for d in data {
            hasher.update(*d);
        }
        hasher.finalize()
    }

    pub fn update(&mut self, v:u64) {
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
    pub fn finalize(&mut self) -> [u64; 4] {
        let starting_one_lane = 1u64;
        let ending_one_lane = 1u64 << 63;
        let one_zero_one_lane = starting_one_lane + ending_one_lane;
        if self.0 == 16 {
            unsafe {
                keccak_push(one_zero_one_lane);
                keccak_new(0u64);
            }
        } else if self.0 < 16 {
            unsafe {
                keccak_push(starting_one_lane);
                for _ in self.0 .. 16 {
                    keccak_push(0);
                }
                keccak_push(ending_one_lane);
            }
        }
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









































































