use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

static GLOBAL_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

pub type PUUID = usize;


// Note: I stole this from snowflake crate
// https://crates.io/crates/snowflake
fn next_global() -> PUUID
{
    let mut prev = GLOBAL_COUNTER.load(Ordering::Relaxed);
    loop
    {
        let old_value = GLOBAL_COUNTER.compare_and_swap(prev, prev + 1, Ordering::Relaxed);
        if old_value == prev
        {
            return prev;
        } else
        {
            prev = old_value;
        }
    }
}

// TODO: 
// Expand PUUID from 1 usize to 4 (akin to a GUUID, 2^128 should be enough different IDs for the
// entirety of the project)
// Create a differentiation between simulation GUUID and Persistent GUUID
pub struct PolyminiUUIDCtx;
impl PolyminiUUIDCtx
{
    pub fn next() -> PUUID 
    {
        next_global()
    }
}
