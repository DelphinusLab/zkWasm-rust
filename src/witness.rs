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
use crate::allocator::start_alloc_witness;
use crate::allocator::stop_alloc_witness;
use crate::allocator::WITNESS_AREA;
use crate::allocator::WITNESS_AREA_END;
use std::mem::size_of;

pub trait WitnessObjWriter {
    fn to_witness(&self, ori_base: *const u8);
}

pub trait WitnessObjReader {
    fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64, base: *const u8);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self);
        }
    }
}

impl WitnessObjReader for u64 {
    fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64, _base: *const u8) {
        *self = fetcher();
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
    fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64, base: *const u8) {
        unsafe {
            let obj = self as *mut Self;
            let arr_ptr = fetcher() as usize;
            let arr_ptr = base.add(arr_ptr);
            let len = fetcher() as usize;
            let cap = fetcher() as usize;
            let obj_ptr = obj as *mut usize;
            *obj_ptr = arr_ptr as usize;
            *obj_ptr.add(1) = len;
            *obj_ptr.add(2) = cap;
            let start = arr_ptr as usize;
            let mem_len = len * size_of::<T>();
            require(start >= WITNESS_AREA);
            require(start <= start + mem_len);
            require(start + mem_len <= WITNESS_AREA_END);
            for i in 0..len {
                //super::dbg!("from witness size of {}\n", l);
                (*(obj_ptr as *mut Vec<T>))[i].from_witness(fetcher, base);
            }
        }
    }
}

fn prepare_witness_obj_inner<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    mut gen: impl FnMut(&T) -> Obj,
    t: &T,
) -> () {
    let b = gen(t);
    let c = Box::new(b.clone());
    let ori_base = get_latest_allocation_base();
    unsafe {
        let diff = (c.as_ref() as *const Obj as *const u8).sub_ptr(ori_base) as u64;
        require(diff == 0);
    }
    c.to_witness(ori_base);
}

pub fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    gen: impl FnMut(&T) -> Obj,
    t: &T,
) -> () {
    unsafe {
        start_alloc_witness();
    }
    prepare_witness_obj_inner(gen, t);
    unsafe {
        stop_alloc_witness();
    }
}

pub fn load_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter>(
    mut fetcher: impl FnMut() -> u64,
    base: *mut u8,
) -> *const Obj {
    let obj_start = base as usize;

    let obj = obj_start as *mut Obj;
    unsafe {
        (*obj).from_witness(&mut fetcher, base);
    }
    obj as *const Obj
}
