use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ALLOCATED_MEMORY: AtomicUsize = AtomicUsize::new(0);

pub struct TrackingAllocator {
    allocator: System,
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            allocator: System,
        }
    }

    pub fn get_allocated() -> usize {
        ALLOCATED_MEMORY.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = self.allocator.alloc(layout);
        if !ret.is_null() {
            ALLOCATED_MEMORY.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);
        ALLOCATED_MEMORY.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}
