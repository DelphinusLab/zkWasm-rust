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

use crate::allocator::alloc_witness_memory;
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
    fn from_witness(&mut self, fetcher: impl Fn() -> u64, base: *const u8);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self);
        }
    }
}

impl WitnessObjReader for u64 {
    fn from_witness(&mut self, fetcher: impl Fn() -> u64, _base: *const u8) {
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
    fn from_witness(&mut self, fetcher: impl Fn() -> u64, base: *const u8) {
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
                (*(obj_ptr as *mut Vec<T>))[i].from_witness(&fetcher, base);
            }
        }
    }
}

fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    gen: impl Fn(&T) -> Obj,
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

fn load_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter>(
    fetcher: impl Fn() -> u64,
    base: *mut u8,
) -> *const Obj {
    let obj_start = base as usize;

    let obj = obj_start as *mut Obj;
    unsafe {
        (*obj).from_witness(fetcher, base);
    }
    obj as *const Obj
}

#[inline(never)]
pub fn prepare_u64_vec(a: i64) {
    unsafe {
        start_alloc_witness();
    }
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
    unsafe {
        stop_alloc_witness();
    }
}

pub fn test_witness_obj() {
    let base_addr = alloc_witness_memory();
    prepare_u64_vec(0);
    let obj = load_witness_obj::<Vec<u64>>(|| unsafe { wasm_witness_pop() }, base_addr);
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

use derive_builder::WitnessObj;

#[derive(WitnessObj, PartialEq, Clone, Debug)]
struct TestA {
    a: u64,
    b: u64,
    c: Vec<u64>,
}

#[inline(never)]
pub fn prepare_test_a(a: i64) {
    unsafe {
        start_alloc_witness();
    }

    prepare_witness_obj(
        |x: &u64| {
            let mut c = vec![];
            for i in 0..10 {
                c.push(*x + (i as u64));
            }
            TestA { a: 1, b: 2, c }
        },
        &(a as u64),
    );
    unsafe {
        stop_alloc_witness();
    }
}

pub fn test_witness_obj_test_a() {
    let base_addr = alloc_witness_memory();
    prepare_test_a(10);
    let obj = load_witness_obj::<TestA>(|| unsafe { wasm_witness_pop() }, base_addr);
    let v = unsafe { &*obj };
    super::dbg!("test a is {:?}\n", v);
}

#[derive(WitnessObj, PartialEq, Clone, Debug)]
struct TestB {
    a: Vec<TestA>,
    c: Vec<u64>,
    b: u64,
}

#[inline(never)]
pub fn prepare_test_b(a: i64) {
    unsafe {
        start_alloc_witness();
    }

    prepare_witness_obj(
        |x: &u64| {
            let mut c = vec![];
            let mut a_array = vec![];
            for _ in 0..3 {
                for i in 0..10 {
                    c.push(*x + (i as u64));
                }
                let a = TestA {
                    a: 1,
                    b: 2,
                    c: c.clone(),
                };
                a_array.push(a);
            }
            TestB {
                a: a_array,
                b: 3,
                c,
            }
        },
        &(a as u64),
    );
    unsafe {
        stop_alloc_witness();
    }
}

pub fn test_witness_obj_test_b() {
    let base_addr = alloc_witness_memory();
    prepare_test_b(0);
    let obj = load_witness_obj::<TestB>(|| unsafe { wasm_witness_pop() }, base_addr);
    let v = unsafe { &*obj };
    super::dbg!("test b is {:?}\n", v);
}
