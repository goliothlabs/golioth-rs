use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use embedded_alloc::Heap;

const HEAP_SIZE: usize = 8192;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

pub fn init() {
    static ONCE: AtomicBool = AtomicBool::new(false);

    // Don't allow init() to be called more than once
    if ONCE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        unsafe {
            HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE);
        }
    }
}
