extern "C" {
    pub fn merkle_setroot(x: u64);
    pub fn merkle_address(x: u64);
    pub fn merkle_set(x: u64);
    pub fn merkle_get() -> u64;
    pub fn merkle_getroot() -> u64;
    pub fn merkle_fetch_data() -> u64;
    pub fn merkle_put_data(x: u64);
}

use crate::kvpair::SMT;
use crate::poseidon::PoseidonHasher;
use crate::require;

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
            LAST_TRACK
                == Some(Track {
                    last_root: root.clone(),
                    last_index: index,
                })
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
            5647113874217112664,
            14689602159481241585,
            4257643359784105407,
            2925219336634521956,
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
        let len = unsafe {
            merkle_address(index as u64);

            merkle_setroot(self.root[0]);
            merkle_setroot(self.root[1]);
            merkle_setroot(self.root[2]);
            merkle_setroot(self.root[3]);

            hash[0] = merkle_get();
            hash[1] = merkle_get();
            hash[2] = merkle_get();
            hash[3] = merkle_get();

            //enforce root does not change
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();

            let len = merkle_fetch_data();
            crate::dbg!("get len is {}\n", len);
            if len > 0 {
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
        };
        Track::set_track(&self.root, index);
        len
    }

    pub fn set(&mut self, index: u32, data: &[u64], pad: bool) {
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
        };
        Track::reset_track();
    }
}

const LEAF_NODE: u64 = 0;
const TREE_NODE: u64 = 1;

fn data_matches_key(data: &[u64], key: &[u64; 4]) -> bool {
    // data[0] == LEAF_NODE
    for i in 0..4 {
        if data[i + 1] != key[i] {
            return false;
        };
    }
    return true;

}

impl Merkle {
    fn smt_get_local(
        &mut self,
        key: &[u64; 4],
        path_index: usize,
        data: &mut [u64],
        pad: bool,
    ) -> u64 {
        crate::dbg!("start smt_get_local {}\n", path_index);
        unsafe { require(path_index < 8) };
        let local_index = (key[path_index / 2] >> (32 * (path_index % 2))) as u32;
        let len = self.get(local_index, data, pad);
        crate::dbg!("smt_get_local key: {:?} local_index: {} datalen: {}\n", key, local_index, len);
        if len == 0 {
            // return len = 0 if no node was find
            return 0
        } else {
            crate::dbg!("smt_get_local with data {:?}\n", data);
            if (data[0] & 0x1) == LEAF_NODE {
                crate::dbg!("smt_get_local is leaf\n");
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
                crate::dbg!("smt_get_local is node\n");
                unsafe { require((data[0] & 0x1) == TREE_NODE) };
                crate::dbg!("pase type check\n");
                let mut sub_merkle = Merkle::load(data[1..5].to_vec().try_into().unwrap());
                sub_merkle.smt_get_local(key, path_index + 1, data, pad)
            }
        }
    }
    fn smt_set_local(&mut self, key: &[u64; 4], path_index: usize, data: &[u64], pad: bool) {
        unsafe { require(path_index < 8) };
        let local_index = (key[path_index / 2] >> (32 * (path_index % 2))) as u32;
        let node_buf = unsafe { DATA_NODE_BUF.as_mut_slice() };
        let len = self.get(local_index, node_buf, pad);
        if len == 0 {
            let data_len = data.len();
            crate::dbg!("smt set local not hit update data {}:\n", data_len);
            let new_data = [[LEAF_NODE].to_vec(), key.to_vec(), data.to_vec()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            self.set(local_index, new_data.as_slice(), pad);
        } else {
            crate::dbg!("smt set local hit:\n");
            if (node_buf[0] & 0x1) == LEAF_NODE {
                crate::dbg!("current node for set is leaf:\n");
                if data_matches_key(node_buf, key) {
                    let len = data.len();
                    crate::dbg!("key match update data {}:\n", len);
                    // if hit the current node
                    let data = Vec::with_capacity(1 + 4 + len);
                    let data = [[LEAF_NODE].to_vec(), key.to_vec(), data.to_vec()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                    self.set(local_index, data.as_slice(), pad);
                } else {
                    crate::dbg!("key not match, creating sub node:\n");
                    // conflict of key here
                    // 1. start a new merkle sub tree
                    let mut sub_merkle = Merkle::new();
                    let existing_node_key: [u64; 4] = node_buf[1..5].to_vec().try_into().unwrap();
                    crate::dbg!("existing node key: {:?}\n", existing_node_key);
                    let existing_node_data = &node_buf[5..len as usize].to_vec();
                    sub_merkle.smt_set_local(&existing_node_key, path_index+1, existing_node_data.as_slice(), pad);
                    sub_merkle.smt_set_local(key, path_index+1, data, pad);
                    let new_data = [
                        TREE_NODE,
                        sub_merkle.root[0],
                        sub_merkle.root[1],
                        sub_merkle.root[2],
                        sub_merkle.root[3],
                    ];
                    // 2 update the current node with the sub merkle tree
                    self.set(local_index, new_data.as_slice(), pad);
                }
            } else {
                // the node is already a sub merkle
                unsafe { require((data[0] & 0x1) == TREE_NODE) };
                let mut sub_merkle = Merkle::load(data[1..4].to_vec().try_into().unwrap());
                sub_merkle.smt_set_local(key, path_index + 1, data, pad);
                let new_data = [
                    TREE_NODE,
                    sub_merkle.root[0],
                    sub_merkle.root[1],
                    sub_merkle.root[2],
                    sub_merkle.root[3],
                ];
                    self.set(local_index, new_data.as_slice(), pad);
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
