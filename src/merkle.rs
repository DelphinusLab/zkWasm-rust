extern "C" {
    pub fn merkle_setroot(x: u64);
    pub fn merkle_address(x: u64);
    pub fn merkle_set(x: u64);
    pub fn merkle_get() -> u64;
    pub fn merkle_getroot() -> u64;
}

use crate::kvpair::SMT;
use crate::poseidon::PoseidonHasher;
use crate::require;
use crate::cache;
use crate::wasm_trace_size;

pub struct Merkle {
    pub root: [u64; 4],
}

#[derive(PartialEq)]
pub struct Track {
    pub last_index: u32,
    pub last_root: [u64; 4],
}

// track the last merkle_root of a merkle_get
static mut LAST_TRACK: Option<Track> = None;
// buf to receive max size of merkle leaf data node
static mut DATA_NODE_BUF: [u64; 1024] = [0; 1024];

impl Track {
    pub fn set_track(root: &[u64; 4], index: u32) {
        unsafe {
            LAST_TRACK = Some(Track {
                last_root: root.clone(),
                last_index: index,
            })
        }
    }

    pub fn reset_track() {
        unsafe { LAST_TRACK = None }
    }

    pub fn tracked(root: &[u64; 4], index: u32) -> bool {
        unsafe {
            match &LAST_TRACK {
                Some(track) => {
                    track.last_root[0] == root[0] &&
                        track.last_root[1] == root[1] &&
                        track.last_root[2] == root[2] &&
                        track.last_root[3] == root[3] &&
                        track.last_index == index
                },
                _=> false
            }
        }
    }
}

impl Merkle {
    /// New Merkle with initial root hash
    /// set root with move to avoid copy
    pub fn load(root: [u64; 4]) -> Self {
        Merkle { root }
    }

    pub fn new() -> Self {
        //THE following is the depth=31, 32 level merkle root default
        let root = [
            14789582351289948625,
            10919489180071018470,
            10309858136294505219,
            2839580074036780766
        ];
        Merkle { root }
    }

    pub fn get_simple(&mut self, index: u32, data: &mut [u64; 4]) {
        unsafe {
            merkle_address(index as u64); // build in merkle address has default depth 32

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            data[0] = merkle_get();
            data[1] = merkle_get();
            data[2] = merkle_get();
            data[3] = merkle_get();

            //enforce root does not change
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();
        }
        Track::set_track(&self.root, index);
    }

    pub fn set_simple(&mut self, index: u32, data: &[u64; 4]) {
        // place a dummy get for merkle proof convension
        if Track::tracked(&self.root, index) {
            ()
        } else {
            unsafe {
                merkle_address(index as u64);

                merkle_setroot(self.root[0]);
                merkle_setroot(self.root[1]);
                merkle_setroot(self.root[2]);
                merkle_setroot(self.root[3]);

                merkle_get();
                merkle_get();
                merkle_get();
                merkle_get();

                //enforce root does not change
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
            }
        }

        unsafe {
            merkle_address(index as u64);

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

        Track::reset_track();
    }

    pub fn get(&mut self, index: u32, data: &mut [u64], pad: bool) -> u64 {
        let mut hash = [0; 4];
        self.get_simple(index, &mut hash);
        let len = cache::fetch_data(&hash, data);
        if len > 0 {
            // FIXME: avoid copy here
            let hash_check = PoseidonHasher::hash(&data[0..len as usize], pad);
            unsafe {
                require(hash[0] == hash_check[0]);
                require(hash[1] == hash_check[1]);
                require(hash[2] == hash_check[2]);
                require(hash[3] == hash_check[3]);
            }
        } else {
            unsafe {
                require(hash[0] == 0);
                require(hash[1] == 0);
                require(hash[2] == 0);
                require(hash[3] == 0);
            }
        }
        len
    }

    pub fn set(&mut self, index: u32, data: &[u64], pad: bool) {
        let trace = unsafe { crate::wasm_trace_size() };
        let hash = PoseidonHasher::hash(data, pad);
        let delta1 = unsafe { crate::wasm_trace_size() - trace };
        cache::store_data(&hash, data);
        let delta2 = unsafe { crate::wasm_trace_size() - trace };
        self.set_simple(index, &hash);
        let delta3 = unsafe { crate::wasm_trace_size() - trace };
        //crate::dbg!("diff {} {} {}\n", delta1, delta2, delta3);
    }
}

const LEAF_NODE: u64 = 0;
const TREE_NODE: u64 = 1;

// internal func: key must have length 4
fn data_matches_key(data: &[u64], key: &[u64]) -> bool {
    // Recall that data[0] == LEAF_NODE
    data[1] == key[0] &&
    data[2] == key[1] &&
    data[3] == key[2] &&
    data[4] == key[3]
    /*
    for i in 0..4 {
        if data[i + 1] != key[i] {
            return false;
        };
    }
    return true;
    */
}

// using a static buf to avoid memory allocation in smt implementation
fn set_smt_data(node_buf: &mut [u64], t: u64, key: &[u64], data: &[u64]) {
    node_buf[0] = t;
    node_buf[1] = key[0];
    node_buf[2] = key[1];
    node_buf[3] = key[2];
    node_buf[4] = key[3];
    unsafe{crate::wasm_dbg(crate::wasm_trace_size())};
    for i in 0..data.len() {
        node_buf[5 + i] = data[i];
    }
    unsafe{crate::wasm_dbg(crate::wasm_trace_size())};
}

impl Merkle {
    fn smt_get_local(
        &mut self,
        key: &[u64; 4],
        path_index: usize,
        data: &mut [u64],
        pad: bool,
    ) -> u64 {
        //crate::dbg!("start smt_get_local {}\n", path_index);
        unsafe { require(path_index < 8) };
        let local_index = (key[path_index / 2] >> (32 * (path_index % 2))) as u32;
        let len = self.get(local_index, data, pad);
        if len == 0 {
            // no node was find
            return 0;
        } else {
            //crate::dbg!("smt_get_local with data {:?}\n", data);
            if (data[0] & 0x1) == LEAF_NODE {
                //crate::dbg!("smt_get_local is leaf\n");
                if data_matches_key(data, key) {
                    for i in 0..(len - 5) {
                        data[i as usize] = data[i as usize + 5]
                    }
                    return len - 5;
                } else {
                    // not hit and return len = 0
                    return 0;
                }
            } else {
                //crate::dbg!("smt_get_local is node: continue in sub merkle\n");
                unsafe { require((data[0] & 0x1) == TREE_NODE) };
                let mut sub_merkle = Merkle::load(data[1..5].try_into().unwrap());
                sub_merkle.smt_get_local(key, path_index + 1, data, pad)
            }
        }
    }

    fn smt_set_local(&mut self, key: &[u64], path_index: usize, data: &[u64], pad: bool) {
        unsafe { require(path_index < 8) };
        let local_index = (key[path_index / 2] >> (32 * (path_index % 2))) as u32;
        let node_buf = unsafe { DATA_NODE_BUF.as_mut_slice() };
        let len = self.get(local_index, node_buf, pad);
        if len == 0 {
            let data_len = data.len();
            //crate::dbg!("smt set local not hit update data {}:\n", data_len);
            unsafe{crate::wasm_dbg(crate::wasm_trace_size())};
            set_smt_data(node_buf, LEAF_NODE, key, data);
            unsafe{crate::wasm_dbg(crate::wasm_trace_size())};
            self.set(local_index, &node_buf[0..5 + data_len], pad);
            unsafe{crate::wasm_dbg(crate::wasm_trace_size())};
        } else {
            //crate::dbg!("smt set local hit:\n");
            if (node_buf[0] & 0x1) == LEAF_NODE {
                //crate::dbg!("current node for set is leaf:\n");
                if data_matches_key(node_buf, key) {
                    let trace = unsafe {wasm_trace_size()};
                    let data_len = data.len();
                    //crate::dbg!("key match update data {}:\n", data_len);
                    // if hit the current node
                    set_smt_data(node_buf, LEAF_NODE, key, data);
                    self.set(local_index, &node_buf[0..5 + data_len], pad);
                    let delta = unsafe {wasm_trace_size() - trace};
                    //crate::dbg!("delta size of set local last hit is {}\n", delta);
                } else {
                    //crate::dbg!("key not match, creating sub node:\n");
                    // conflict of key here
                    // 1. start a new merkle sub tree
                    let mut sub_merkle = Merkle::new();
                    sub_merkle.smt_set_local(
                        &node_buf[1..5],
                        path_index + 1,
                        &node_buf[5..len as usize],
                        pad,
                    );
                    sub_merkle.smt_set_local(key, path_index + 1, data, pad);
                    set_smt_data(node_buf, TREE_NODE, sub_merkle.root.as_slice(), &[]);
                    // 2 update the current node with the sub merkle tree
                    self.set(local_index, &node_buf[0..5], pad);
                }
            } else {
                //crate::dbg!("current node for set is node:\n");
                // the node is already a sub merkle
                unsafe { require((node_buf[0] & 0x1) == TREE_NODE) };
                let mut sub_merkle = Merkle::load(node_buf[1..5].try_into().unwrap());
                sub_merkle.smt_set_local(key, path_index + 1, data, pad);
                set_smt_data(node_buf, TREE_NODE, sub_merkle.root.as_slice(), &[]);
                self.set(local_index, &node_buf[0..5], pad);
            }
        }
    }
}

impl SMT for Merkle {
    fn smt_get(&mut self, key: &[u64; 4], data: &mut [u64], pad: bool) -> u64 {
        self.smt_get_local(key, 0, data, pad)
    }

    fn smt_set(&mut self, key: &[u64; 4], data: &[u64], pad: bool) {
        self.smt_set_local(key, 0, data, pad)
    }
}
