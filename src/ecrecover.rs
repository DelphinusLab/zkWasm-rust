use crate::ecrecover_finalize;
use crate::ecrecover_new;
use crate::ecrecover_push;
use primitive_types::U256;

pub fn ecrecover(sig_r: &[u64], sig_s: &[u64], z: &[u64], r_point: &[u64]) -> (U256, u64) {
    unsafe {
        ecrecover_new(1u64);
    }
    for v in sig_r
        .iter()
        .chain(sig_s.iter())
        .chain(z.iter())
        .chain(r_point.iter())
    {
        unsafe {
            ecrecover_push(*v);
        }
    }

    unsafe {
        (
            U256([
                ecrecover_finalize(),
                ecrecover_finalize(),
                ecrecover_finalize(),
                ecrecover_finalize(),
            ]),
            ecrecover_finalize(),
        )
    }
}
