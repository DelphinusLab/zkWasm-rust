pub trait SMT {
    fn smt_get(&mut self, key: &[u64; 4], data: &mut [u64], pad: bool) -> u64;
    fn smt_set(&mut self, key: &[u64; 4], data: &[u64], pad: bool);
}

/// sparse merkle tree implemented by adding indicators at leafs of each group (32 depth)
/// to indicate whether the leaf is a data leaf or a root of a deeper merkle tree
pub struct KeyValueMap<S: SMT> {
    merkle: S,
    pad: bool,
}

impl<S: SMT> KeyValueMap<S> {
    pub fn new(root_merkle: S, pad: bool) -> Self {
        KeyValueMap {
            merkle: root_merkle,
            pad,
        }
    }
    pub fn set(&mut self, key: &[u64; 4], data_buf: &[u64]) {
        self.merkle.smt_set(key, data_buf, self.pad);
    }
    pub fn get(&mut self, key: &[u64; 4], data_buf: &mut [u64]) -> u64 {
        self.merkle.smt_get(key, data_buf, self.pad)
    }
}
