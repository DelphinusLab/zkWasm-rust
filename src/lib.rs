extern "C" {
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn require(cond:i32);
    pub fn log(v:u64);

    fn kvpair_setroot(x:u64);
    fn kvpair_address(x:u64);
    fn kvpair_set(x:u64);
    fn kvpair_get() -> u64;
    fn kvpair_getroot() -> u64;
}

pub struct Merkle {
    pub root: [u64; 4]
}

impl Merkle {
    pub fn load(root: [u64; 4]) -> Self {
        unsafe {
            kvpair_setroot(root[0]);
            kvpair_setroot(root[1]);
            kvpair_setroot(root[2]);
            kvpair_setroot(root[3]);
        }
        Merkle { root }
    }

    pub fn new() -> Self {
        //TODO: fix the hardcoded height 20 merkle root
        let root = [4074723173704310182, 3116368985344895753, 15689180094961269493, 694055158784170088];
        unsafe {
            kvpair_setroot(root[0]);
            kvpair_setroot(root[1]);
            kvpair_setroot(root[2]);
            kvpair_setroot(root[3]);
        }
        Merkle { root }
    }

    pub fn get(index: u64, data: &mut [u64; 4]) {
        unsafe {
            kvpair_address(index);
            data[0] = kvpair_get();
            data[1] = kvpair_get();
            data[2] = kvpair_get();
            data[3] = kvpair_get();
        }
    }

    pub fn getroot(data: &mut [u64; 4]) {
        unsafe {
            data[0] = kvpair_getroot();
            data[1] = kvpair_getroot();
            data[2] = kvpair_getroot();
            data[3] = kvpair_getroot();
        }
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


use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn zkmain() -> i64 {
    let _data = vec![0x83, b'c', b'a', b't'];
    0
}


