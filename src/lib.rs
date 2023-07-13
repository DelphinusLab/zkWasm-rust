extern "C" {
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn require(cond:i32);
    pub fn wasm_dbg(v:u64);

    pub fn kvpair_setroot(x:u64);
    pub fn kvpair_address(x:u64);
    pub fn kvpair_set(x:u64);
    pub fn kvpair_get() -> u64;
    pub fn kvpair_getroot() -> u64;
    pub fn poseidon_new(x:u64);
    pub fn poseidon_push(x:u64);
    pub fn poseidon_finalize() -> u64;

    pub fn babyjubjub_sum_new(x:u64);
    pub fn babyjubjub_sum_push(x:u64);
    pub fn babyjubjub_sum_finalize() -> u64;

}

pub struct Merkle {}

impl Merkle {
    pub fn load(root: &[u64; 4]) {
        unsafe {
            kvpair_setroot(root[0]);
            kvpair_setroot(root[1]);
            kvpair_setroot(root[2]);
            kvpair_setroot(root[3]);
        }
    }

    pub fn new() {
        //TODO: fix the hardcoded height 20 merkle root
        let root = [4074723173704310182, 3116368985344895753, 15689180094961269493, 694055158784170088];
        unsafe {
            kvpair_setroot(root[0]);
            kvpair_setroot(root[1]);
            kvpair_setroot(root[2]);
            kvpair_setroot(root[3]);
        }
    }

    pub fn get(index: u64) -> [u64; 4] {
        let mut data = [0; 4];
        unsafe {
            kvpair_address(index);
            data[0] = kvpair_get();
            data[1] = kvpair_get();
            data[2] = kvpair_get();
            data[3] = kvpair_get();
        }
        data
    }

    pub fn getroot() -> [u64; 4] {
        let mut data = [0; 4];
        unsafe {
            data[0] = kvpair_getroot();
            data[1] = kvpair_getroot();
            data[2] = kvpair_getroot();
            data[3] = kvpair_getroot();
        }
        data
    }

    pub fn set(index: u64, data: &[u64; 4]) {
        unsafe {
            kvpair_address(index);
            kvpair_set(data[0]);
            kvpair_set(data[1]);
            kvpair_set(data[2]);
            kvpair_set(data[3]);
        }
    }

    pub fn setroot(data: &[u64; 4]) {
        unsafe {
            kvpair_setroot(data[0]);
            kvpair_setroot(data[1]);
            kvpair_setroot(data[2]);
            kvpair_setroot(data[3]);
        }
    }
}

pub struct PoseidonHasher (u64);

impl PoseidonHasher {
    pub fn new() -> Self {
        unsafe {
            poseidon_new(1u64);
        }
        PoseidonHasher(0u64)
    }
    pub fn update(&mut self, v:u64) {
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
        for _ in (self.0 & 0x3) .. 4 {
            unsafe {
                poseidon_push(0);
            }
            self.0 += 1;
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
        for _ in self.0 .. 32 {
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

pub struct BabyJubjubPoint {
    x: [u64;4],
    y: [u64;4],
}

impl BabyJubjubPoint {
    pub fn msm(points: Vec<(&BabyJubjubPoint, &[u64; 4])>) -> BabyJubjubPoint {
        let mut len = points.len();
        unsafe {
            babyjubjub_sum_new(1u64);
        }
        for (point, scalar) in points {
            unsafe {
                babyjubjub_sum_push(point.x[0]);
                babyjubjub_sum_push(point.x[1]);
                babyjubjub_sum_push(point.x[2]);
                babyjubjub_sum_push(point.x[3]);
                babyjubjub_sum_push(point.y[0]);
                babyjubjub_sum_push(point.y[1]);
                babyjubjub_sum_push(point.y[2]);
                babyjubjub_sum_push(point.y[3]);
                babyjubjub_sum_push(scalar[0]);
                babyjubjub_sum_push(scalar[1]);
                babyjubjub_sum_push(scalar[2]);
                babyjubjub_sum_push(scalar[3]);
                len -= 1;
                if len != 0 {
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_new(0u64);
                }
            }
        }
        unsafe {
            BabyJubjubPoint {
                x: [
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                ],
                y: [
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                ],
            }
        }
    }
}

pub struct JubjubSignature {
    pub sig_r: BabyJubjubPoint,
    pub neg_sig_s: [u64; 4],
}

// 0 = c . pk + R - S . P_G that requires all points to be in the same group
// let lhs = vk.mul_scalar(&c).add(&sig_r);
// let rhs = p_g.mul_scalar(&sig_s);


impl JubjubSignature {
    pub fn verify(&self, pk: &BabyJubjubPoint, base: &BabyJubjubPoint, msghash: &[u64; 4]) {
        unsafe {
            let r = BabyJubjubPoint::msm(vec![
                (pk, msghash),
                (&self.sig_r, &[1,0,0,0]),
                (base, &self.neg_sig_s),
            ]);
            require(if r.x == [0,0,0,0] { 1 } else {0} );
            require(if r.y == [1,0,0,0] { 1 } else {0} );
        }
    }
}


#[cfg(feature = "test")]


mod test {
    extern "C" {
        pub fn wasm_input(is_public: u32) -> u64;
        pub fn require(cond:bool);
        pub fn wasm_dbg(v:u64);

        pub fn kvpair_setroot(x:u64);
        pub fn kvpair_address(x:u64);
        pub fn kvpair_set(x:u64);
        pub fn kvpair_get() -> u64;
        pub fn kvpair_getroot() -> u64;
        pub fn poseidon_new(x:u64);
        pub fn poseidon_push(x:u64);
        pub fn poseidon_finalize() -> u64;
    }

    use super::BabyJubjubPoint;
    use super::JubjubSignature;


    use wasm_bindgen::prelude::*;
    use super::PoseidonHasher;
    #[wasm_bindgen]
    pub fn zkmain() -> i64 {
        let mut hasher = PoseidonHasher::new();
        let data = vec![0x1, 0, 0];
        for d in data {
            hasher.update(d);
        }
        let z = hasher.finalize();
        /*
        unsafe {
            require(z[0] == 1);
        }
        */

        let c = BabyJubjubPoint {x:[0,0,0,0], y:[1, 0, 0, 0]};
        let p = BabyJubjubPoint::msm(vec![(&c, &[1,0,0,0])]);

        unsafe {
            require(p.x[0] == 0);
            require(p.y[0] == 1);
        }

        let base = BabyJubjubPoint {
            x: [0x6adb52fc9ee7a82c,
                0x9de555e0ba6a693c,
                0x9bc0d49fa725bddf,
                0x274dbce8d1517996],
            y: [0x4595febfd51eb853,
                0xb2e78246231640b5,
                0xe2eae9a542bd99f6,
                0x5ce98c61b05f47f]
        };

        let sig = JubjubSignature {
            sig_r : BabyJubjubPoint {
                x: [0x79f2349047d5c157,
                    0xc88fee14d607cbe7,
                    0x6e35bc47bd9afe6c,
                    0x2491aba8d3a191a7],
                y: [0x348dd8f7f99152d7,
                    0xf9a9d4ed0cb0c1d1,
                    0x18dbddfd24c35583,
                    0x2e07297f8d3c3d78]
            },
            neg_sig_s: [0,0,0,0]
        };

        let pk = BabyJubjubPoint {
            x: [0x9067a2afaebaf361,
                0x72dded51978190e1,
                0xb3b811eaacd0ec7c,
                0x11805510440a3488],
            y:  [0xa100dd448e072c13,
                 0xa89a9027777af9f,
                 0xf9ff77744a39298a,
                 0x1f07aa1b3c598e2f]
        };

        sig.verify(&pk, &base, &[0,0,0,0]);
        0
    }
}
