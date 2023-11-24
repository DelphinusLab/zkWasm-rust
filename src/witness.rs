extern "C" {
    /// inserts a witness at the current wasm_private inputs cursor
    pub fn wasm_witness_insert(u: u64);
    pub fn wasm_witness_pop() -> u64;
    pub fn wasm_input(x: u32) -> u64;
    pub fn require(cond: bool);
    pub fn wasm_dbg(v: u64);
}

use std::alloc::{GlobalAlloc, Layout, System};
use std::mem::size_of;
use std::ptr::null_mut;

static mut WITNESS_AREA: usize = 0;
static mut WITNESS_AREA_END: usize = 0;
const MAX_WITNESS_OBJ_SIZE: usize = 80 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;

static mut ALLOC_WITNESS: bool = false;
static mut SIMPLE_ALLOCATOR: SimpleAllocator = SimpleAllocator {
    area: 0usize as *mut u8,
    remaining: MAX_WITNESS_OBJ_SIZE,
};

/// Alloca a block of memory to hold the witness object.
pub fn alloc_witness_memory() -> *mut u8 {
    let base_addr =
        unsafe { std::alloc::alloc(Layout::from_size_align(MAX_WITNESS_OBJ_SIZE, 8).unwrap()) };
    unsafe {
        WITNESS_AREA = base_addr as usize;
        WITNESS_AREA_END = WITNESS_AREA + MAX_WITNESS_OBJ_SIZE;
    };
    base_addr
}

struct SimpleAllocator {
    pub area: *mut u8,
    remaining: usize,
}

struct HybridAllocator {}

unsafe fn start_alloc_witness() {
    SIMPLE_ALLOCATOR.area =
        unsafe { std::alloc::alloc(Layout::from_size_align(MAX_WITNESS_OBJ_SIZE, 8).unwrap()) };
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
            SIMPLE_ALLOCATOR.area.add(SIMPLE_ALLOCATOR.remaining)
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
    fn from_witness(obj: *mut Self);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
        unsafe {
            wasm_witness_insert(*self);
        }
    }
}

impl WitnessObjReader for u64 {
    fn from_witness(obj: *mut Self) {
        unsafe {
            *obj = wasm_witness_pop();
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
    fn from_witness(obj: *mut Self) {
        unsafe {
            let arr_ptr = wasm_witness_pop() as usize;
            let len = wasm_witness_pop() as usize;
            let cap = wasm_witness_pop() as usize;
            let obj_ptr = obj as *mut usize;
            *obj_ptr = arr_ptr;
            *obj_ptr.add(1) = len;
            *obj_ptr.add(2) = cap;
            let offset = arr_ptr as *mut T;
            let start = arr_ptr as usize;
            let mem_len = len * size_of::<T>();
            require(start >= WITNESS_AREA);
            require(mem_len < MAX_WITNESS_OBJ_SIZE);
            require(start + len <= WITNESS_AREA_END);
            for i in 0..len {
                //T::from_witness(unsafe { offset.add(i) as *mut T });
                *(offset as *mut u64).add(i) = wasm_witness_pop();
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
    let ori_base = unsafe { SIMPLE_ALLOCATOR.area.add(SIMPLE_ALLOCATOR.remaining) };
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
    Obj::from_witness(obj);
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

#[inline(never)]
pub fn prepare_u64_vec(base: *const u8, a: i64) {
    prepare_witness_obj(
        base,
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
    unsafe { wasm_dbg(base_addr as u64) };
    let obj = load_witness_obj::<Vec<u64>, u64>(base_addr, |base| prepare_u64_vec(base, 0));
    let v = unsafe { &*obj };
    for i in 0..100 {
        unsafe {
            //wasm_dbg(v[i]);
            require(v[i] == (i as u64))
        };
    }
}

use derive_builder::WitnessObj;

#[derive (WitnessObj)]
struct testA {
    Aa: u64,
    Ab: u64,
    Ac: Vec<u64>,
}

#[derive (WitnessObj)]
struct testB {
    Ba: testA,
    Bc: Vec<u64>,
    Bb: u64,
}