extern "C" {
    pub fn cache_set_mode(x: u64);
    pub fn cache_set_hash(x: u64);
    pub fn cache_store_data(x: u64);
    pub fn cache_fetch_data() -> u64;
}

// It is better for the following to be phantom if data has large size
pub fn store_data(hash: &[u64; 4], data: &[u64]) {
    unsafe {
        cache_set_mode(1);
        for i in 0..data.len() {
            cache_store_data(data[i]);
        }
        cache_set_hash(hash[0]);
        cache_set_hash(hash[1]);
        cache_set_hash(hash[2]);
        cache_set_hash(hash[3]);
    }
}

// It is better for the following to be phantom if data has large size
pub fn fetch_data(hash: &[u64; 4], data: &mut [u64]) -> u64 {
    unsafe {
        cache_set_mode(0);
        cache_set_hash(hash[0]);
        cache_set_hash(hash[1]);
        cache_set_hash(hash[2]);
        cache_set_hash(hash[3]);
        let len = cache_fetch_data();
        if len > 0 {
            crate::require(len <= data.len() as u64);
            for i in 0..len as usize {
                data[i] = cache_fetch_data();
            }
        }
        return len;
    }
}

/// lagency fetch_data witch returns a vec instead of writing directly into a buf
pub fn get_data(hash: &[u64; 4]) -> Vec<u64> {
    unsafe {
        cache_set_mode(0);
        cache_set_hash(hash[0]);
        cache_set_hash(hash[1]);
        cache_set_hash(hash[2]);
        cache_set_hash(hash[3]);
        let len = cache_fetch_data();
        if len > 0 {
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len as usize {
                data.push(cache_fetch_data());
            }
            data
        } else {
            vec![]
        }
    }
}
