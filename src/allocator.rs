use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::null_mut;

pub static mut WITNESS_AREA: usize = 0;
pub static mut WITNESS_AREA_END: usize = 0;
pub const MAX_WITNESS_OBJ_SIZE: usize = 40 * 1024;
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

pub fn get_base_ptr() -> *mut u8 {
    unsafe {
        let base_addr = SIMPLE_ALLOCATOR.area.as_ptr();
        base_addr as *mut u8
    }
}

pub fn get_cursor_ptr() -> *mut u8 {
    unsafe {
        let base_addr = SIMPLE_ALLOCATOR.area.as_ptr();
        (base_addr as *mut u8).add(SIMPLE_ALLOCATOR.remaining)
    }
}

struct SimpleAllocator {
    pub area: [u8; MAX_WITNESS_OBJ_SIZE],
    remaining: usize,
}

struct HybridAllocator {}

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

pub unsafe fn start_alloc_witness() {
    ALLOC_WITNESS = true;
}

pub unsafe fn stop_alloc_witness() {
    ALLOC_WITNESS = false;
    SIMPLE_ALLOCATOR.remaining = MAX_WITNESS_OBJ_SIZE;
}
