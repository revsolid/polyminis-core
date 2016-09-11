use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

static GLOBAL_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

fn next_global() -> usize {
    let mut prev = GLOBAL_COUNTER.load(Ordering::Relaxed);
    loop {
        let old_value = GLOBAL_COUNTER.compare_and_swap(prev, prev + 1, Ordering::Relaxed);
        if old_value == prev {
            return prev;
        } else {
            prev = old_value;
        }
    }
}

pub struct PolyminiUUIDCtx;
impl PolyminiUUIDCtx
{
    pub fn next() -> usize 
    {
        next_global()
    }
}
