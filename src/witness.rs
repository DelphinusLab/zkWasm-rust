extern "C" {
    /// injects a witness at the current wasm_private inputs cursor
    pub fn wasm_witness_inject(u: u64);
    pub fn wasm_witness_pop() -> u64;
    pub fn wasm_input(x: u32) -> u64;
    pub fn require(cond: bool);
    pub fn wasm_dbg(v: u64);
}
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::mem::size_of;
use std::ptr::null_mut;

static mut WITNESS_AREA: usize = 0;
static mut WITNESS_AREA_END: usize = 0;
const MAX_WITNESS_OBJ_SIZE: usize = 80 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;

struct SimpleAllocator {
    pub area: *mut u8,
    remaining: usize,
}

struct HybridAllocator {}

static mut ALLOC_WITNESS: bool = false;
static mut SIMPLE_ALLOCATOR: SimpleAllocator = SimpleAllocator {
    area: 0usize as *mut u8,
    remaining: MAX_WITNESS_OBJ_SIZE,
};

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
    fn to_witness(&self, ori_base: *const u8, wit_base: *const u8, writer: impl Fn(u64));
}

pub trait WitnessObjReader {
    fn from_witness(obj: *mut Self);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8, writer: impl Fn(u64)) {
        writer(*self);
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
    fn to_witness(&self, ori_base: *const u8, wit_base: *const u8, writer: impl Fn(u64)) {
        let c: &[usize; 3] = unsafe { std::mem::transmute(self) };
        let arr_ptr = unsafe { wit_base.add((c[0] as *const u8).sub_ptr(ori_base)) };
        writer(arr_ptr as u64);
        writer(c[1] as u64);
        writer(c[2] as u64);
        for t in self {
            t.to_witness(ori_base, wit_base, &writer);
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
            unsafe {
                require(start >= WITNESS_AREA);
                require(mem_len < MAX_WITNESS_OBJ_SIZE);
                require(start + len <= WITNESS_AREA_END);
            }
            for i in 0..len {
                //T::from_witness(unsafe { offset.add(i) as *mut T });
                *(offset as *mut u64).add(i) = wasm_witness_pop();
            }
        }
    }
}

pub fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    base: *const u8,
    gen: impl Fn(&T) -> Obj,
    t: &T,
    writer: impl Fn(u64),
) -> () {
    let b = gen(t);
    let c = Box::new(b.clone());
    let ori_base = unsafe { SIMPLE_ALLOCATOR.area.add(SIMPLE_ALLOCATOR.remaining) };
    unsafe {
        writer((c.as_ref() as *const Obj as *const u8).sub_ptr(ori_base) as u64);
    }
    c.to_witness(ori_base, base, writer);
}

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[inline(never)]
pub fn prepare_u64_vec(base: *const u8, mut a: usize) {
    if a != 0 {
        prepare_u64_vec(base, a - 2);
    } else {
        unsafe {
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
                |x: u64| unsafe { wasm_witness_inject(x) },
            );
        }
    }
}

pub fn load_witness_obj_inner<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    base: *mut u8,
    t: &T,
    writer: impl Fn(u64),
    reader: impl Fn() -> u64,
    prepare: impl FnOnce(*const u8, usize),
) -> *const Obj {
    unsafe {
        start_alloc_witness();
    }
    let mut a = unsafe { wasm_input(0) as usize };
    unsafe {
        wasm_dbg(a as u64);
    }
    prepare(base, a);
    unsafe {
        stop_alloc_witness();
    }

    let obj_offset = reader() as usize;
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

fn load_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(
    base: *mut u8,
    t: &T,
    prepare: impl FnOnce(*const u8, usize),
) -> *const Obj {
    let obj = load_witness_obj_inner(
        base,
        t,
        |x: u64| unsafe { wasm_witness_inject(x) },
        || unsafe { wasm_witness_pop() },
        prepare,
    );
    obj
}

#[cfg(test)]
mod tests {
    use crate::witness::load_witness_obj_inner;
    use crate::witness::MAX_WITNESS_OBJ_SIZE;
    use std::cell::UnsafeCell;

    static mut UARRAY: Vec<u64> = vec![];
    /*
    #[derive (Clone)]
    struct WObj {
        a: u64,
        b: u64,
        //array: Box<Vec<u32>>
        array: Vec<u32>
    }
    */

    #[test]
    fn test_alloc() {
        let base = UnsafeCell::new([0x55; MAX_WITNESS_OBJ_SIZE]);
        let base_addr = base.get().cast::<u64>();
        unsafe {
            WITNESS_AREA = base_addr as usize;
            WITNESS_AREA_END = WITNESS_AREA + MAX_WITNESS_OBJ_SIZE;
        }
        println!("witness base addr is {:?}", base_addr);
        let obj = load_witness_obj_inner(
            base_addr as *mut u64,
            |x: &u64| {
                let mut a = vec![];
                for i in 0..100 {
                    a.push(*x + (i as u64));
                }
                a
            },
            &32,
            |w| unsafe {
                println!("push {}", w);
                UARRAY.insert(0, w)
            },
            || unsafe {
                println!("pop");
                UARRAY.pop().unwrap()
            },
            prepare_u64_vec,
        );
        let v = unsafe { &*obj };
        for i in 0..100 {
            assert!(v[i] == 32u64 + (i as u64));
        }
        println!("obj result is {:?}", v);
    }
}

pub fn test_witness_obj() {
    /*
    #[derive (Clone)]
    struct WObj {
        a: u64,
        b: u64,
        array: Vec<u32>
    }
    */
    //unsafe { wasm_input(0) };
    let base_addr =
        unsafe { std::alloc::alloc(Layout::from_size_align(MAX_WITNESS_OBJ_SIZE, 8).unwrap()) };

    unsafe {
        WITNESS_AREA = base_addr as usize;
        WITNESS_AREA_END = WITNESS_AREA + MAX_WITNESS_OBJ_SIZE;
    }

    let obj = load_witness_obj::<Vec<u64>, u64>(base_addr, &32, prepare_u64_vec);
    /*
    let v = unsafe { &*obj };
    for i in 0..100 {
        unsafe {
            //wasm_dbg(123454321);
            //wasm_dbg(v[i]);
        };
    }

    for i in 0..100 {
        unsafe {
            //wasm_dbg(i as u64);
            //wasm_dbg(v[i]);
            //require(v[i] == 32u64 + (i as u64))
        };
    } */
}
