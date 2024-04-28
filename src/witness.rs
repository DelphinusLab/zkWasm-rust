extern "C" {
    /// inserts a witness at the current wasm_private inputs cursor
    pub fn wasm_witness_insert(u: u64);
    pub fn wasm_witness_pop() -> u64;
    pub fn wasm_witness_set_index(x: u64);
    pub fn wasm_witness_indexed_pop() -> u64;
    pub fn wasm_witness_indexed_insert(x: u64);
    pub fn wasm_witness_indexed_push(x: u64);
    pub fn require(cond: bool);
}

use crate::allocator::get_latest_allocation_base;
use crate::allocator::WITNESS_AREA;
use crate::allocator::WITNESS_AREA_END;
use crate::allocator::stop_alloc_witness;
use crate::allocator::start_alloc_witness;
use crate::allocator::alloc_witness_memory;
use std::mem::size_of;

pub trait WitnessFetcher {
    fn get_witness() -> u64;
}

pub struct InputFetcher {}

impl WitnessFetcher for InputFetcher {
    fn get_witness() -> u64 {
        unsafe {crate::wasm_input(0)}
    }
}

pub struct DynamicWitnessFetcher {}

impl WitnessFetcher for DynamicWitnessFetcher {
    fn get_witness() -> u64 {
        unsafe {wasm_witness_pop()}
    }
}

pub trait WitnessObjWriter {
    fn to_witness(&self, ori_base: *const u8);
}

pub trait WitnessObjReader {
    fn from_witness<WF: WitnessFetcher>(obj: *mut Self, base: *const u8);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self);
        }
    }
}

impl WitnessObjReader for u64 {
    fn from_witness<WF: WitnessFetcher>(obj: *mut Self, _base: *const u8) {
        unsafe {
            *obj = WF::get_witness();
        }
    }
}

impl<T: WitnessObjWriter> WitnessObjWriter for Vec<T> {
    /// [ptr, len, capacity, array[0... self.len()]
    fn to_witness(&self, ori_base: *const u8) {
        let c: &[usize; 3] = unsafe { std::mem::transmute(self) };
        let arr_ptr = unsafe { (c[0] as *const u8).sub_ptr(ori_base) };
        unsafe {
            wasm_witness_insert(arr_ptr as u64);
            wasm_witness_insert(c[1] as u64);
            wasm_witness_insert(c[2] as u64);
        }
        for t in self {
            t.to_witness(ori_base);
        }
    }
}

impl<T: WitnessObjReader> WitnessObjReader for Vec<T> {
    fn from_witness<WF: WitnessFetcher>(obj: *mut Self, base: *const u8) {
        unsafe {
            let arr_ptr = WF::get_witness() as usize;
            let arr_ptr = base.add(arr_ptr);
            let len = WF::get_witness() as usize;
            let cap = WF::get_witness() as usize;
            let obj_ptr = obj as *mut usize;
            *obj_ptr = arr_ptr as usize;
            *obj_ptr.add(1) = len;
            *obj_ptr.add(2) = cap;
            let offset = arr_ptr as *mut T;
            let start = arr_ptr as usize;
            let mem_len = len * size_of::<T>();
            require(start >= WITNESS_AREA);
            require(start <= start + mem_len);
            require(start + mem_len <= WITNESS_AREA_END);
            for i in 0..len {
                T::from_witness::<WF>(offset.add(i) as *mut T, base);
            }
        }
    }
}

fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    gen: impl Fn(&T) -> Obj,
    t: &T,
) -> () {
    unsafe {
        start_alloc_witness();
    }
    let b = gen(t);
    let c = Box::new(b.clone());
    let ori_base = get_latest_allocation_base();
    unsafe {
        let diff = (c.as_ref() as *const Obj as *const u8).sub_ptr(ori_base) as u64;
        require(diff == 0);
    }
    c.to_witness(ori_base);
    unsafe {
        stop_alloc_witness();
    }
}

fn load_witness_obj<WF: WitnessFetcher, Obj: Clone + WitnessObjReader + WitnessObjWriter>(
    base: *mut u8,
) -> *const Obj {
    let obj_start = base as usize;
    unsafe {
        require(obj_start >= WITNESS_AREA);
    }
    let obj = obj_start as *mut Obj;
    Obj::from_witness::<WF>(obj, base);
    obj as *const Obj
}

#[inline(never)]
pub fn prepare_u64_vec(a: i64) {
    prepare_witness_obj(
        |x: &u64| {
            let mut a = vec![];
            for i in 0..2000 {
                a.push(*x + (i as u64));
            }
            a
        },
        &(a as u64),
    );
}

pub fn test_witness_obj() {
    let base_addr = alloc_witness_memory();
    crate::dbg!("witness base addr is {:?}\n", base_addr);
    prepare_u64_vec(0);
    let obj = load_witness_obj::<DynamicWitnessFetcher, Vec<u64>>(base_addr);
    let v = unsafe { &*obj };
    for i in 0..100 {
        unsafe { require(v[i] == (i as u64)) };
    }
}

pub fn test_witness_indexed(i: u64) {
    unsafe {
        wasm_witness_set_index(i);
        wasm_witness_indexed_push(0x0);
        wasm_witness_indexed_push(0x1);
        wasm_witness_indexed_insert(0x2);
        let a = wasm_witness_indexed_pop();
        require(a == 0x1);
        let a = wasm_witness_indexed_pop();
        require(a == 0x0);
        let a = wasm_witness_indexed_pop();
        require(a == 0x2);
    }
}
