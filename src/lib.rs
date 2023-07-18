use primitive_types::U256;

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
        let root = [253654092113440498, 968977278742622784, 6195234659416948485, 820733412028077155];
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

#[derive(Debug, Clone, PartialEq)]
pub struct BabyJubjubPoint {
    pub x: U256,
    pub y: U256,
}

pub const MODULUS: [u64; 4] = [
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

pub fn negative_of_fr(b: &[u64; 4]) -> [u64; 4] {
    let mut borrow = 0;
    let mut a = MODULUS.clone();
    for i in 0..4 {
        if a[i] - borrow < b[i] {
            a[i] += (u64::MAX - b[i]) + 1 - borrow;
            borrow = 1
        } else {
            a[i] -= b[i] + borrow;
            borrow = 0;
        }
    };
    a
}

impl BabyJubjubPoint {
    pub fn msm(points: Vec<(&BabyJubjubPoint, &[u64; 4])>) -> BabyJubjubPoint {
        let mut len = points.len();
        unsafe {
            babyjubjub_sum_new(1u64);
        }
        for (point, scalar) in points {
            unsafe {
                babyjubjub_sum_push(point.x.0[0]);
                babyjubjub_sum_push(point.x.0[1]);
                babyjubjub_sum_push(point.x.0[2]);
                babyjubjub_sum_push(point.x.0[3]);
                babyjubjub_sum_push(point.y.0[0]);
                babyjubjub_sum_push(point.y.0[1]);
                babyjubjub_sum_push(point.y.0[2]);
                babyjubjub_sum_push(point.y.0[3]);
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
                x: U256([
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                ]),
                y: U256([
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                   babyjubjub_sum_finalize(),
                ]),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JubjubSignature {
    pub sig_r: BabyJubjubPoint,
    pub sig_s: [u64; 4],
}

// 0 = c . pk + R - S . P_G that requires all points to be in the same group
// let lhs = vk.mul_scalar(&c).add(&sig_r);
// let rhs = p_g.mul_scalar(&sig_s);

const NEG_BASE:BabyJubjubPoint = BabyJubjubPoint {
            x: U256([5098030607081443850, 11739138394996609992, 7617911478965053006, 103675969630295906]),
            y: U256([10973966134842004663, 8445032247919564157, 8665528646177973254, 405343104476405055]),
        };

impl JubjubSignature {
    pub fn verify(&self, pk: &BabyJubjubPoint, msghash: &[u64; 4]) {
        unsafe {
            let r = BabyJubjubPoint::msm(vec![
                (pk, msghash),
                (&self.sig_r, &[1,0,0,0]),
                (&NEG_BASE, &self.sig_s),
            ]);
            require(r.x == U256([0,0,0,0]));
            require(r.y == U256([1,0,0,0]));
        }
    }
}


#[cfg(feature = "test")]


mod test {
    extern "C" {
        pub fn wasm_input(is_public: u32) -> u64;
        pub fn require(cond: bool);
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
    use super::Merkle;
    use primitive_types::U256;


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

        Merkle::new();

        Merkle::set(0, &[1,1,2,2]);
        let c = Merkle::get(0);
        unsafe {
            require(c == [1,1,2,2]);
        }

        let c = BabyJubjubPoint {x:U256([0,0,0,0]), y:U256([1, 0, 0, 0])};
        let p = BabyJubjubPoint::msm(vec![(&c, &[1,0,0,0])]);

        unsafe {
            require(p.x.0[0] == 0);
            require(p.y.0[0] == 1);
        }


        let sig = JubjubSignature {
            sig_r : BabyJubjubPoint {
                x: U256([3942246333445170378, 4927712981048651912, 7483524259745080053, 60536396037540871]),
                y: U256([14850245140538961756, 11076552477444376689, 6805567804001881962, 3473463521075824379]),
            },
            sig_s: [13068069613806562103, 2598268142923890778, 9227627411507601187, 303022261472651166]
        };

        let pk = BabyJubjubPoint {
            x: U256([7885996749535148040, 5452996086172756687, 10631572794003595355, 1413880906945024417]),
            y: U256([13330009580783412631, 14458870954835491754, 9623332966787297474, 160649411381582638]),
        };

        sig.verify(&pk, &[32195221423877958,0,0,0]);
        0
    }
}
