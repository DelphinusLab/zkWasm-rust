use primitive_types::U256;

extern "C" {
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn wasm_output(v: u64);
    pub fn wasm_read_context() -> u64;
    pub fn wasm_write_context(v: u64);
    pub fn require(cond:bool);
    pub fn wasm_dbg(v:u64);

    pub fn merkle_setroot(x:u64);
    pub fn merkle_address(x:u64);
    pub fn merkle_set(x:u64);
    pub fn merkle_get() -> u64;
    pub fn merkle_getroot() -> u64;
    pub fn merkle_fetch_data() -> u64;
    pub fn merkle_put_data(x:u64);
    pub fn poseidon_new(x:u64);
    pub fn poseidon_push(x:u64);
    pub fn poseidon_finalize() -> u64;

    pub fn babyjubjub_sum_new(x:u64);
    pub fn babyjubjub_sum_push(x:u64);
    pub fn babyjubjub_sum_finalize() -> u64;

}

pub struct Merkle {
    pub root: [u64; 4]
}

impl Merkle {
    /// New Merkle with initial root hash
    /// set root with move to avoid copy
    pub fn load(root: [u64; 4]) -> Self {
        Merkle { root }
    }

    pub fn new() -> Self {
        //THE following is the depth=31, 32 level merkle root default
        let root = [11826054925775482837, 5943555147602679911, 3550282808714298530, 3363170373529648096];
        Merkle { root }
    }

    pub fn get_simple(&self, index: u64, data: &mut [u64; 4]) {
        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            data[0] = merkle_get();
            data[1] = merkle_get();
            data[2] = merkle_get();
            data[3] = merkle_get();
        }
    }

    pub fn set_simple(&mut self, index: u64, data: &[u64; 4]) {
        // place a dummy get for merkle proof convension
        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            merkle_get();
            merkle_get();
            merkle_get();
            merkle_get();
        }

        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            merkle_set(data[0]);
            merkle_set(data[1]);
            merkle_set(data[2]);
            merkle_set(data[3]);

            self.root[0] = merkle_getroot();
            self.root[1] = merkle_getroot();
            self.root[2] = merkle_getroot();
            self.root[3] = merkle_getroot();
        }
    }


    pub fn get(&self, index: u64, data: &mut [u64], pad: bool) -> u64 {
        let mut hash = [0; 4];
        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            hash[0] = merkle_get();
            hash[1] = merkle_get();
            hash[2] = merkle_get();
            hash[3] = merkle_get();

            let len = merkle_fetch_data();
            if len>0 {
                require(len <= data.len() as u64);
                for i in 0..len {
                    data[i as usize] = merkle_fetch_data();
                }

                // FIXME: avoid copy here
                let hash_check = PoseidonHasher::hash(&data[0..len as usize], pad);
                require(hash[0] == hash_check[0]);
                require(hash[1] == hash_check[1]);
                require(hash[2] == hash_check[2]);
                require(hash[3] == hash_check[3]);
            }
            len
        }
    }


    pub fn set(&mut self, index: u64, data: &[u64], pad: bool) {
        // place a dummy get for merkle proof convension
        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            merkle_get();
            merkle_get();
            merkle_get();
            merkle_get();
        }

        unsafe {
            merkle_address(index);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            for d in data.iter() {
                merkle_put_data(*d);
            }

            let hash = PoseidonHasher::hash(data, pad);
            merkle_set(hash[0]);
            merkle_set(hash[1]);
            merkle_set(hash[2]);
            merkle_set(hash[3]);

            self.root[0] = merkle_getroot();
            self.root[1] = merkle_getroot();
            self.root[2] = merkle_getroot();
            self.root[3] = merkle_getroot();
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
    pub fn hash(data: &[u64], padding: bool) -> [u64; 4] {
        let mut hasher = Self::new();
        if padding {
            let group = data.len() / 3;
            let mut j = 0;
            for i in 0..group {
                j = i*3;
                hasher.update(data[j]);
                hasher.update(data[j+1]);
                hasher.update(data[j+2]);
                hasher.update(0u64);
            }
            j += 3;
            for i in j..data.len() {
                hasher.update(data[i]);
            }
        } else {
            for d in data {
                hasher.update(*d);
            }
        }
        hasher.finalize()
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
        if (self.0 & 0x3) != 0 {
            for _ in (self.0 & 0x3) .. 4 {
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
        let data = vec![0x1, 0x1, 2, 2];
        for d in data {
            hasher.update(d);
        }
        let z = hasher.finalize();
        /*
        unsafe {
            require(z[0] == 1);
        }
        */

        let mut merkle = Merkle::new();
        let mut leaf = [0,0,0,0];

        merkle.set(0, &[1,1,2,2], false);

        let len = merkle.get(0, &mut leaf, false);

        unsafe {
            require(len == 4);
            require(leaf == [1,1,2,2]);
        }


        merkle.set(0, &[3,4,5,6,7], true);
        let mut leaf = [0,0,0,0,0];

        let len = merkle.get(0, &mut leaf, true);

        unsafe {
            require(len == 5);
            require(leaf == [3,4,5,6,7]);
        }

        merkle.set_simple(1, &[4,5,6,7]);
        let mut leaf2 = [0,0,0,0];

        merkle.get_simple(1, &mut leaf2);

        unsafe {
            require(leaf2 == [4,5,6,7]);
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
