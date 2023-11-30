extern "C" {
    /// inserts a witness at the current wasm_private inputs cursor
    pub fn wasm_witness_insert(u: u64);
    pub fn wasm_witness_pop() -> u64;
}

use crate::require;
use primitive_types::U256;
use std::alloc::{GlobalAlloc, Layout, System};
use std::mem::size_of;
use std::ptr::null_mut;

static mut WITNESS_AREA: usize = 0;
static mut WITNESS_AREA_END: usize = 0;
const MAX_WITNESS_OBJ_SIZE: usize = 40 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;

static mut ALLOC_WITNESS: bool = false;
static mut SIMPLE_ALLOCATOR: SimpleAllocator = SimpleAllocator {
    area: [0; MAX_WITNESS_OBJ_SIZE],
    remaining: MAX_WITNESS_OBJ_SIZE,
};

/// SETUP for WITNESS_AREA and WITNESS_AREA_END
pub fn init_simple_allocator() -> *mut u8 {
    unsafe {
        let base_addr = SIMPLE_ALLOCATOR.area.as_ptr();
        WITNESS_AREA = base_addr as usize;
        WITNESS_AREA_END = WITNESS_AREA + MAX_WITNESS_OBJ_SIZE;
        base_addr as *mut u8
    }
}

struct SimpleAllocator {
    pub area: [u8; MAX_WITNESS_OBJ_SIZE],
    remaining: usize,
}

struct HybridAllocator {}

unsafe fn start_alloc_witness() {
    ALLOC_WITNESS = true;
}

unsafe fn stop_alloc_witness() {
    ALLOC_WITNESS = false;
    SIMPLE_ALLOCATOR.remaining = MAX_WITNESS_OBJ_SIZE;
}

unsafe impl Sync for HybridAllocator {}

#[global_allocator]
static ALLOCATOR: HybridAllocator = HybridAllocator {};

unsafe impl GlobalAlloc for HybridAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOC_WITNESS {
            let size = layout.size();
            let align = layout.align();
            let align_mask_to_round_down = !(align - 1);
            if align > MAX_SUPPORTED_ALIGN {
                return null_mut();
            }
            if size > SIMPLE_ALLOCATOR.remaining {
                return null_mut();
            }
            SIMPLE_ALLOCATOR.remaining -= size;
            SIMPLE_ALLOCATOR.remaining &= align_mask_to_round_down;
            let area_ptr = SIMPLE_ALLOCATOR.area.as_ptr() as *mut u8;
            area_ptr.add(SIMPLE_ALLOCATOR.remaining)
        } else {
            System.alloc(layout)
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ALLOC_WITNESS {
        } else {
            System.dealloc(ptr, layout);
        }
    }
}

pub trait WitnessObjWriter {
    fn to_witness(&self, ori_base: *const u8, wit_base: *const u8);
}

pub trait WitnessObjReader {
    fn from_witness(&mut self);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self);
        }
    }
}

impl WitnessObjReader for u64 {
    fn from_witness(&mut self) {
        unsafe {
            *self = wasm_witness_pop();
        }
    }
}

impl WitnessObjWriter for i64 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self as u64);
        }
    }
}

impl WitnessObjReader for i64 {
    fn from_witness(&mut self) {
        unsafe {
            *self = wasm_witness_pop() as i64;
        }
    }
}

impl WitnessObjWriter for u32 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self as u64);
        }
    }
}

impl WitnessObjReader for u32 {
    fn from_witness(&mut self) {
        unsafe {
            *self = wasm_witness_pop() as u32;
        }
    }
}

impl WitnessObjWriter for u128 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            let words: [u64; 2] = std::mem::transmute::<u128, [u64; 2]>(*self);
            wasm_witness_insert(words[0]);
            wasm_witness_insert(words[1]);
        }
    }
}

impl WitnessObjReader for u128 {
    fn from_witness(&mut self) {
        unsafe {
            let words = [wasm_witness_pop(), wasm_witness_pop()];
            let v = std::mem::transmute::<[u64; 2], u128>(words);
            *self = v;
        }
    }
}

impl WitnessObjWriter for U256 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            wasm_witness_insert(self.0[0]);
            wasm_witness_insert(self.0[1]);
            wasm_witness_insert(self.0[2]);
            wasm_witness_insert(self.0[3]);
        }
    }
}

impl WitnessObjReader for U256 {
    fn from_witness(&mut self) {
        unsafe {
            self.0[0] = wasm_witness_pop();
            self.0[1] = wasm_witness_pop();
            self.0[2] = wasm_witness_pop();
            self.0[3] = wasm_witness_pop();
        }
    }
}

impl<T: WitnessObjWriter> WitnessObjWriter for Vec<T> {
    /// [ptr, len, capacity, array[0... self.len()]
    fn to_witness(&self, ori_base: *const u8, wit_base: *const u8) {
        let c: &[usize; 3] = unsafe { std::mem::transmute(self) };
        let arr_ptr = unsafe { wit_base.add((c[0] as *const u8).sub_ptr(ori_base)) };
        unsafe {
            wasm_witness_insert(arr_ptr as u64);
            wasm_witness_insert(c[1] as u64);
            wasm_witness_insert(c[2] as u64);
        }
        for t in self {
            t.to_witness(ori_base, wit_base);
        }
    }
}

impl<T: WitnessObjReader> WitnessObjReader for Vec<T> {
    fn from_witness(&mut self) {
        unsafe {
            let obj = self as *mut Self;
            let arr_ptr = wasm_witness_pop() as usize;
            let len = wasm_witness_pop() as usize;
            let cap = wasm_witness_pop() as usize;

            //super::dbg!("read arr_ptr is {:?} {:?} {:?}\n", arr_ptr, len, cap);
            let obj_ptr = obj as *mut usize;
            *obj_ptr = arr_ptr;
            *obj_ptr.add(1) = len;
            *obj_ptr.add(2) = cap;
            let start = arr_ptr as usize;
            let mem_len = len * size_of::<T>();
            //super::dbg!("start is {} WITNESS_AREA is {}\n", start, WITNESS_AREA);

            require(start >= WITNESS_AREA);
            require(mem_len < MAX_WITNESS_OBJ_SIZE);
            require(start + len <= WITNESS_AREA_END);
            for i in 0..len {
                (*(obj_ptr as *mut Vec<T>))[i].from_witness();
            }
        }
    }
}

fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    base: *const u8,
    gen: impl Fn(&T) -> Obj,
    t: &T,
) -> () {
    let b = gen(t);
    let c = Box::new(b.clone());
    let ori_base = unsafe {
        let area = SIMPLE_ALLOCATOR.area.as_ptr() as *mut u8;
        area.add(SIMPLE_ALLOCATOR.remaining)
    };
    unsafe {
        wasm_witness_insert((c.as_ref() as *const Obj as *const u8).sub_ptr(ori_base) as u64);
    }
    c.to_witness(ori_base, base);
}

fn load_witness_obj_inner<Obj: Clone + WitnessObjReader + WitnessObjWriter>(
    base: *mut u8,
    prepare: impl FnOnce(*const u8),
) -> *const Obj {
    unsafe {
        start_alloc_witness();
    }
    prepare(base);
    unsafe {
        stop_alloc_witness();
    }

    let obj_offset = unsafe { wasm_witness_pop() as usize };
    let obj_start = base as usize + obj_offset;
    let obj_end = obj_start + obj_offset;
    unsafe {
        require(obj_start >= WITNESS_AREA);
        require(obj_end <= WITNESS_AREA_END);
    }
    let obj = obj_start as *mut Obj;
    unsafe {
        (*obj).from_witness();
    }
    obj as *const Obj
}

/// Load an object into wasm witness queue and restore it back to address start at (base: *mut 8)
fn load_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    base: *mut u8,
    prepare: impl FnOnce(*const u8),
) -> *const Obj {
    let obj = load_witness_obj_inner(base, prepare);
    obj
}

#[cfg(feature = "zktest")]
pub(crate) mod test {
    use super::init_simple_allocator;
    use super::load_witness_obj;
    use super::prepare_witness_obj;
    use super::WitnessObjReader;
    use super::WitnessObjWriter;
    use crate::require;
    use crate::wasm_dbg;
    use derive_builder::WitnessObj;

    #[inline(never)]
    pub fn prepare_u64_vec(base: *const u8, a: i64) {
        prepare_witness_obj(
            base,
            |x: &u64| {
                let mut a = vec![];
                for i in 0..200 {
                    a.push(*x + (i as u64));
                }
                a
            },
            &(a as u64),
        );
    }

    pub fn test_witness_obj() {
        let base_addr = init_simple_allocator();
        let obj = load_witness_obj::<Vec<u64>, u64>(base_addr, |base| prepare_u64_vec(base, 0));
        let v = unsafe { &*obj };
        for i in 0..100 {
            unsafe { require(v[i] == (i as u64)) };
        }
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    struct TestA {
        a: u64,
        b: u128,
        c: Vec<u64>,
    }

    #[inline(never)]
    pub fn prepare_test_a(base: *const u8, a: i64) {
        prepare_witness_obj(
            base,
            |x: &u64| {
                let mut c = vec![];
                for i in 0..10 {
                    c.push(*x + (i as u64));
                }
                TestA { a: 1, b: 2, c }
            },
            &(a as u64),
        );
    }

    pub fn test_witness_obj_test_a() {
        let base_addr = init_simple_allocator();
        unsafe { wasm_dbg(base_addr as u64) };
        let obj = load_witness_obj::<TestA, usize>(base_addr, |base| prepare_test_a(base, 10));
        let v = unsafe { &*obj };
        crate::dbg!("test a is {:?}\n", v);
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    struct TestB {
        a: Vec<TestA>,
        c: Vec<u32>,
        b: u128,
    }

    #[inline(never)]
    pub fn prepare_test_b(base: *const u8, a: u32) {
        prepare_witness_obj(
            base,
            |x: &u32| {
                let mut c = vec![];
                let mut a_array = vec![];
                for _ in 0..3 {
                    for i in 0..10 {
                        c.push(*x + (i as u32));
                    }
                    let a = TestA {
                        a: 1,
                        b: 2 << 64,
                        c: c.iter().map(|x| *x as u64).collect::<Vec<_>>(),
                    };
                    a_array.push(a);
                }
                TestB {
                    a: a_array,
                    b: 3 << 64,
                    c,
                }
            },
            &(a as u32),
        );
    }

    pub fn test_witness_obj_test_b() {
        let base_addr = init_simple_allocator();
        unsafe { wasm_dbg(base_addr as u64) };
        let obj = load_witness_obj::<TestB, usize>(base_addr, |base| prepare_test_b(base, 0));
        let v = unsafe { &*obj };
        crate::dbg!("test b is {:?}\n", v);
    }
}
