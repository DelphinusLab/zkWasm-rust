extern "C" {
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn require(cond:i32);
    pub fn wasm_dbg(v:u64);

    fn kvpair_setroot(x:u64);
    fn kvpair_address(x:u64);
    fn kvpair_set(x:u64);
    fn kvpair_get() -> u64;
    fn kvpair_getroot() -> u64;
    fn poseidon_new(x:u64);
    fn poseidon_push(x:u64);
    fn poseidon_finalize() -> u64;
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


#[cfg(feature = "test")]


mod test {
    extern "C" {
        pub fn wasm_input(is_public: u32) -> u64;
        pub fn require(cond:bool);
        pub fn wasm_dbg(v:u64);

        fn kvpair_setroot(x:u64);
        fn kvpair_address(x:u64);
        fn kvpair_set(x:u64);
        fn kvpair_get() -> u64;
        fn kvpair_getroot() -> u64;
        fn poseidon_new(x:u64);
        fn poseidon_push(x:u64);
        fn poseidon_finalize() -> u64;
    }


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
        unsafe {
            require(z[0] == 1);
        }
        0
    }
}
