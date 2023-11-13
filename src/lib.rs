#![feature(ptr_sub_ptr)]

extern "C" {
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn wasm_output(v: u64);
    pub fn wasm_read_context() -> u64;
    pub fn wasm_write_context(v: u64);
    pub fn require(cond: bool);
    pub fn wasm_dbg(v: u64);

    pub fn merkle_setroot(x: u64);
    pub fn merkle_address(x: u64);
    pub fn merkle_set(x: u64);
    pub fn merkle_get() -> u64;
    pub fn merkle_getroot() -> u64;
    pub fn merkle_fetch_data() -> u64;
    pub fn merkle_put_data(x: u64);
    pub fn poseidon_new(x: u64);
    pub fn poseidon_push(x: u64);
    pub fn poseidon_finalize() -> u64;

    pub fn babyjubjub_sum_new(x: u64);
    pub fn babyjubjub_sum_push(x: u64);
    pub fn babyjubjub_sum_finalize() -> u64;

}

pub mod merkle;
pub mod poseidon;
pub mod jubjub;
pub mod witness;

#[cfg(feature = "test")]
mod test;
