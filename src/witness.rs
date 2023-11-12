extern "C" {
    /// injects a witness at the current wasm_private inputs cursor
    pub fn wasm_witness(u: u64);
    pub fn wasm_input(is_public: u32) -> u64;
    pub fn require(cond: bool);
    pub fn wasm_dbg(v: u64);
}
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::ptr::null_mut;


const MAX_WITNESS_OBJ_SIZE: usize = 1 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;


struct SimpleAllocator {
    area: UnsafeCell<[u8; MAX_WITNESS_OBJ_SIZE]>,
    remaining: usize
}

struct HybridAllocator {}

static mut ALLOC_WITNESS: bool = false;
static mut SIMPLE_ALLOCATOR: SimpleAllocator =
    SimpleAllocator {
        area: UnsafeCell::new([0x55; MAX_WITNESS_OBJ_SIZE]),
        remaining: MAX_WITNESS_OBJ_SIZE,
    };

unsafe fn start_alloc_witness() {
    ALLOC_WITNESS = true;
}

unsafe fn stop_alloc_witness() {
    ALLOC_WITNESS = false;
    SIMPLE_ALLOCATOR.remaining = MAX_WITNESS_OBJ_SIZE;
}

unsafe impl Sync for HybridAllocator {}

#[global_allocator]
static ALLOCATOR: HybridAllocator= HybridAllocator {};

unsafe impl GlobalAlloc for HybridAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOC_WITNESS {
            let size = layout.size();
            let align = layout.align();
            let align_mask_to_round_down = !(align - 1);
            if align > MAX_SUPPORTED_ALIGN {
                return null_mut();
            }
            if size >  SIMPLE_ALLOCATOR.remaining {
                return null_mut();
            }
            SIMPLE_ALLOCATOR.remaining -= size;
            SIMPLE_ALLOCATOR.remaining &= align_mask_to_round_down;
            SIMPLE_ALLOCATOR.area.get().cast::<u8>().add(SIMPLE_ALLOCATOR.remaining)
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
    fn to_witness(&self, ori_base: *const u64, wit_base: *const u64, writer: impl Fn (u64));
}

pub trait WitnessObjReader {
    fn from_witness(base: *mut u64, reader: impl Fn () -> u64);
}

impl WitnessObjWriter for u64 {
    fn to_witness(&self, _ori_base: *const u64, _wit_base: *const u64, writer: impl Fn (u64)) {
        writer(*self);
    }
}

impl WitnessObjReader for u64 {
    fn from_witness(wit_base: *mut u64, reader: impl Fn()->u64) {
        unsafe { *wit_base = reader(); }
    }
}



impl<T:WitnessObjWriter> WitnessObjWriter for Vec<T> {
    fn to_witness(&self, ori_base: *const u64, wit_base: *const u64, writer: impl Fn (u64)) {
        let c = unsafe {
            std::mem::transmute::<&Vec<T>, &[u64;9]>(self)
        };
        let offset = unsafe { (c[0] as *const u64).sub_ptr(ori_base) };
        writer(unsafe{wit_base.add(offset) as u64});
        writer(c[1]);
        writer(c[2]);
        for t in self {
            t.to_witness(ori_base, wit_base, &writer);
        }
    }
}

impl<T:WitnessObjReader> WitnessObjReader for Vec<T> {
    fn from_witness(wit_base: *mut u64, reader: impl Fn()->u64) {
        let p = reader();
        let len = reader();
        let cap = reader();
        unsafe {
            *wit_base = p;
            *(wit_base.add(1)) = len;
            *(wit_base.add(2)) = cap;
        } 
        let offset = unsafe { wit_base.add(3) as *const T };
        //println!("witness base is {:?}, witness obj address is {}", wit_base, p);
        for i in 0..len {
            T::from_witness(unsafe {offset.add(i as usize) as *mut u64}, &reader);
        }
    }
}

pub fn prepare_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(base: *const u64, gen: impl Fn(&T) -> Obj, t:&T, writer: impl Fn(u64)) -> () {
    let b = gen(t);
    let c = Box::new(b.clone());
    let ori_base = unsafe {SIMPLE_ALLOCATOR.area.get().cast::<u8>().add(SIMPLE_ALLOCATOR.remaining)};
    //println!("ori base is {:?}", ori_base);
    c.to_witness(ori_base as *const u64, base, writer);
}

fn load_witness_obj_inner<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(base: *mut u64, gen: impl Fn(&T) -> Obj, t:&T, writer: impl Fn(u64), reader: impl Fn()->u64) -> *const Obj {
    unsafe {start_alloc_witness();}
    prepare_witness_obj(base, gen, t, writer);
    unsafe {stop_alloc_witness();}
    Obj::from_witness(base, reader);
    let obj = unsafe {
        std::mem::transmute::<*const u64, *const Obj>(base)
    };
    obj
}

fn load_witness_obj<Obj: Clone + WitnessObjReader + WitnessObjWriter, T>(base: *mut u64, gen: impl Fn (&T) -> Obj, t:&T) -> *const Obj {
    load_witness_obj_inner(
        base,
        gen,
        t,
        |x| {
            unsafe {wasm_witness(x)}
        },
        || {
            unsafe {wasm_input(0)}
        }
    )
}

#[cfg(test)]
mod tests {
    use crate::witness::MAX_WITNESS_OBJ_SIZE;
    use crate::witness::load_witness_obj_inner;
    use std::cell::UnsafeCell;

    static mut UARRAY:Vec<u64> = vec![];
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
        println!("witness base addr is {:?}", base_addr);
        let obj = load_witness_obj_inner(base_addr as *mut u64, |_x:&u64| {
            vec![101,102,103,104,105]
        }, &32, |w| unsafe { 
            println!("push {}", w);
            UARRAY.insert(0, w) 
        }, || unsafe {
            println!("pop");
            UARRAY.pop().unwrap()
        });
        let v = unsafe { &*obj };
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

        unsafe { wasm_dbg(1234 as u64); }
        let base = UnsafeCell::new([0x55; MAX_WITNESS_OBJ_SIZE]);
        let base_addr = base.get().cast::<u64>();
        unsafe { wasm_dbg(base_addr as u64); }

        let obj = load_witness_obj(base_addr as *mut u64, |x:&u64| {
            let mut v = vec![];
            for i in 0..1 {
                v.push(x + i);
            };
            v
        }, &32);
        let v = unsafe { &*obj };
        unsafe {require(v[0] == 32)};
}

