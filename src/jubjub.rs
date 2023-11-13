use primitive_types::U256;
use crate::babyjubjub_sum_new;
use crate::babyjubjub_sum_push;
use crate::babyjubjub_sum_finalize;
use crate::require;

#[derive(Debug, Clone, PartialEq)]
pub struct BabyJubjubPoint {
    pub x: U256,
    pub y: U256,
}

pub const MODULUS: [u64; 4] = [
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

pub fn negative_of_fr(b: &[u64; 4]) -> [u64; 4] {
    let mut borrow = 0;
    let mut a = MODULUS.clone();
    for i in 0..4 {
        if a[i] - borrow < b[i] {
            a[i] += (u64::MAX - b[i]) + 1 - borrow;
            borrow = 1
        } else {
            a[i] -= b[i] + borrow;
            borrow = 0;
        }
    }
    a
}

impl BabyJubjubPoint {
    pub fn msm(points: &[(&BabyJubjubPoint, &[u64; 4])]) -> BabyJubjubPoint {
        let mut len = points.len();
        unsafe {
            babyjubjub_sum_new(1u64);
        }
        for (point, scalar) in points {
            unsafe {
                babyjubjub_sum_push(point.x.0[0]);
                babyjubjub_sum_push(point.x.0[1]);
                babyjubjub_sum_push(point.x.0[2]);
                babyjubjub_sum_push(point.x.0[3]);
                babyjubjub_sum_push(point.y.0[0]);
                babyjubjub_sum_push(point.y.0[1]);
                babyjubjub_sum_push(point.y.0[2]);
                babyjubjub_sum_push(point.y.0[3]);
                babyjubjub_sum_push(scalar[0]);
                babyjubjub_sum_push(scalar[1]);
                babyjubjub_sum_push(scalar[2]);
                babyjubjub_sum_push(scalar[3]);
                len -= 1;
                if len != 0 {
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_finalize();
                    babyjubjub_sum_new(0u64);
                }
            }
        }
        unsafe {
            BabyJubjubPoint {
                x: U256([
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                ]),
                y: U256([
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                    babyjubjub_sum_finalize(),
                ]),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JubjubSignature {
    pub sig_r: BabyJubjubPoint,
    pub sig_s: [u64; 4],
}

// 0 = c . pk + R - S . P_G that requires all points to be in the same group
// let lhs = vk.mul_scalar(&c).add(&sig_r);
// let rhs = p_g.mul_scalar(&sig_s);

const NEG_BASE: BabyJubjubPoint = BabyJubjubPoint {
    x: U256([
        5098030607081443850,
        11739138394996609992,
        7617911478965053006,
        103675969630295906,
    ]),
    y: U256([
        10973966134842004663,
        8445032247919564157,
        8665528646177973254,
        405343104476405055,
    ]),
};

const ONE: U256 = U256([1, 0, 0, 0]);

impl JubjubSignature {
    pub fn verify(&self, pk: &BabyJubjubPoint, msghash: &[u64; 4]) {
        unsafe {
            let r = BabyJubjubPoint::msm(&[
                (pk, msghash),
                (&self.sig_r, &ONE.0),
                (&NEG_BASE, &self.sig_s),
            ]);
            require(r.x.is_zero() && r.y == ONE);
        }
    }
}

