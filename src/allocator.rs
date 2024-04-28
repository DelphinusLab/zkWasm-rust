use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::null_mut;

const MAX_SUPPORTED_ALIGN: usize = 4096;
const MAX_WITNESS_OBJ_SIZE: usize = 80 * 1024;

pub static mut WITNESS_AREA: usize = 0;
pub static mut WITNESS_AREA_END: usize = 0;

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

pub unsafe fn start_alloc_witness() {
    SIMPLE_ALLOCATOR.area =
        unsafe { std::alloc::alloc(Layout::from_size_align(MAX_WITNESS_OBJ_SIZE, 8).unwrap()) };
    ALLOC_WITNESS = true;
}

pub unsafe fn stop_alloc_witness() {
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

pub fn get_latest_allocation_base() -> *const u8 {
    unsafe { SIMPLE_ALLOCATOR.area.add(SIMPLE_ALLOCATOR.remaining) }
}
