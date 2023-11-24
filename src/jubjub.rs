use crate::babyjubjub_sum_finalize;
use crate::babyjubjub_sum_new;
use crate::babyjubjub_sum_push;
use crate::require;
use primitive_types::U256;
use std::ops::SubAssign;
use std::ops::MulAssign;
use std::ops::AddAssign;
use std::ops::{Mul, Sub, Add};
use std::ops::Neg;
use crate::ff::Field;
use crate::ff::PrimeField;

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

    pub fn to_serialized(&self) -> U256 {
        let mut r = self.y;
        if self.x.0[0] & 0x1 == 0x1 {
            r.0[3] |= 0x8000000000000000u64;
        }
        r
    }

    pub fn from_serialized(v: U256) -> Self {
        use crate::bn256::Fr;
        use crate::bn256::repr_from_u256;
        use crate::bn256::repr_to_u256;
        let mut y_repr = <Fr as PrimeField>::Repr::default();
        let mut s = v;
        s.0[3] &= 0x7fffffffffffffffu64;
        repr_from_u256(y_repr.as_mut(), &s.0);
        let y = <Fr as PrimeField>::from_repr(y_repr).unwrap();
        let y2 = y.square(); // y^2
        let mut y2d = y2;
        let mut d = Fr::from(168696u64);
        d.mul_assign(Fr::from(168700u64).invert().unwrap());
        let d = d.neg();
        y2d.mul_assign(d); // dy^2
        y2d.add_assign(Fr::ONE); //dy^2 + 1
        let mut x = y2;
        x.sub_assign(Fr::ONE);  //y^2 - 1
        //x = x.neg(); // 1 - y^2
        x.mul_assign(y2d.invert().unwrap()); // y^2-1/1+dy^2
        x = x.sqrt().unwrap();
        let v = v.0[3];
        let sign = (v & 0x8000000000000000u64) != 0;
        if bool::from(x.is_odd()) != sign {
            x = x.neg();
        }
        BabyJubjubPoint {
            x: U256(repr_to_u256(x.to_repr())),
            y: s,
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
