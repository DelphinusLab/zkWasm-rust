extern "C" {
    pub fn merkle_setroot(x: u64);
    pub fn merkle_address(x: u64);
    pub fn merkle_set(x: u64);
    pub fn merkle_get() -> u64;
    pub fn merkle_getroot() -> u64;
    pub fn merkle_fetch_data() -> u64;
    pub fn merkle_put_data(x: u64);
}

use crate::require;
use crate::poseidon::PoseidonHasher;

pub struct Merkle {
    pub root: [u64; 4],
}

#[derive (PartialEq)]
pub struct Track {
    pub last_index: u64,
    pub last_root: [u64; 4],
}

// track the last merkle_root of a merkle_get
static mut LAST_TRACK: Option<Track> = None;

impl Track {
    pub fn set_track(root: &[u64; 4], index: u64) {
        unsafe {
            LAST_TRACK = Some (Track {
                last_root: root.clone(),
                last_index: index,
            })
        }
    }

    pub fn reset_track() {
        unsafe {
            LAST_TRACK = None
        }
    }

    pub fn tracked(root: &[u64; 4], index: u64) -> bool {
        unsafe {
            LAST_TRACK == Some (Track {
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

    pub fn get_simple(&mut self, index: u64, data: &mut [u64; 4]) {
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

            //enforce root does not change
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();
            merkle_getroot();
        }
        Track::set_track(&self.root, index);
    }

    pub fn set_simple(&mut self, index: u64, data: &[u64; 4]) {
        // place a dummy get for merkle proof convension
        if Track::tracked(&self.root, index) {
            ()
        } else {
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

                //enforce root does not change
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
            }
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

        Track::reset_track();
    }

    pub fn get(&mut self, index: u64, data: &mut [u64], pad: bool) -> u64 {
        let mut hash = [0; 4];
        let len = unsafe {
            merkle_address(index);

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

    pub fn set(&mut self, index: u64, data: &[u64], pad: bool) {
        if Track::tracked(&self.root, index) {
            ()
        } else {
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

                //enforce root does not change
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
                merkle_getroot();
            }
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
        };
        Track::reset_track();
    }
}
