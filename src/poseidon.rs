use crate::poseidon_finalize;
use crate::poseidon_new;
use crate::poseidon_push;

pub struct PoseidonHasher(u64);

impl PoseidonHasher {
    pub fn new() -> Self {
        unsafe {
            poseidon_new(1u64);
        }
        PoseidonHasher(0u64)
    }
    pub fn hash(data: &[u64], padding: bool) -> [u64; 4] {
        let mut hasher = Self::new();
        if padding {
            let group = data.len() / 3;
            let mut j = 0;
            for i in 0..group {
                j = i * 3;
                hasher.update_pad_field(&data[j..j+3]);
            }
            j += 3;
            for i in j..data.len() {
                hasher.update(data[i]);
            }
            let len = data.len();
        } else {
            for d in data {
                hasher.update(*d);
            }
        }
        hasher.finalize()
    }

    // for better trace size
    fn update_pad_field(&mut self, v: &[u64]) {
        unsafe {
            poseidon_push(v[0]);
            poseidon_push(v[1]);
            poseidon_push(v[2]);
            poseidon_push(0u64);
        }
        self.0 += 4;
        if self.0 == 32 {
            unsafe {
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_new(0u64);
            }
            self.0 = 0;
        }
    }

    pub fn update(&mut self, v: u64) {
        unsafe {
            poseidon_push(v);
        }
        self.0 += 1;
        if self.0 == 32 {
            unsafe {
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_new(0u64);
            }
            self.0 = 0;
        }
    }
    pub fn finalize(&mut self) -> [u64; 4] {
        if (self.0 & 0x3) != 0 {
            for _ in (self.0 & 0x3)..4 {
                unsafe {
                    poseidon_push(0);
                }
                self.0 += 1;
            }
        }
        if self.0 == 32 {
            unsafe {
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_finalize();
                poseidon_new(0u64);
            }
            self.0 = 0;
        }
        unsafe {
            poseidon_push(1);
        }
        self.0 += 1;
        for _ in self.0..32 {
            unsafe {
                poseidon_push(0);
            }
        }
        unsafe {
            [
                poseidon_finalize(),
                poseidon_finalize(),
                poseidon_finalize(),
                poseidon_finalize(),
            ]
        }
    }
}
