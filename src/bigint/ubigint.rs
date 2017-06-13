use std::cmp::{self, Ordering, PartialEq, PartialOrd, Eq, Ord};

use super::algorithms::{self, Limb, LIMB_BITS};
use super::montgomery;
use utils::slice_ext::SliceExt;

#[derive(Clone)]
pub struct UBigInt {
    limbs: Box<[Limb]>,
    bits: u32
}

impl UBigInt {
    pub fn new(bits: u32) -> Self {
        assert!(bits > 0);
        UBigInt {
            // Ensure there's an extra high bit available, since intermediate
            // products may be slightly larger than the final result.
            limbs: vec![0; (bits/LIMB_BITS + 1) as usize].into_boxed_slice(),
            bits: bits
        }
    }
    pub fn bits(&self) -> u32 {
        self.bits
    }
    pub fn set_u32(&mut self, value: u32) {
        self.limbs[1..].fill_copy(0);
        self.limbs[0] = value;
    }
    pub fn set(&mut self, other: &UBigInt) {
        let n = cmp::min(self.limbs.len(), other.limbs.len());
        self.limbs[0..n].copy_from_slice(&other.limbs[0..n]);
        self.limbs[n..].fill_copy(0);
    }
    pub fn mul_add(&mut self, a: &UBigInt, b: &UBigInt) {
        algorithms::mul_add(&mut self.limbs, &a.limbs, &b.limbs);
    }
    pub fn mul_sub(&mut self, a: &UBigInt, b: &UBigInt) {
        algorithms::mul_sub(&mut self.limbs, &a.limbs, &b.limbs);
    }
    pub fn add(&mut self, a: &UBigInt) {
        algorithms::add(&mut self.limbs, &a.limbs);
    }
    pub fn sub(&mut self, a: &UBigInt) {
        algorithms::sub(&mut self.limbs, &a.limbs);
    }
    pub fn div_mod(&mut self, a: &UBigInt, out: &mut UBigInt) {
        algorithms::div_mod(&mut out.limbs, &mut self.limbs, &a.limbs);
    }
    pub fn pure_mod(&mut self, a: &UBigInt) {
        algorithms::pure_mod(&mut self.limbs, &a.limbs);
    }
    pub fn shl(&mut self, n: u32) {
        algorithms::shl(&mut self.limbs, n);
    }
    pub fn shr(&mut self, n: u32) {
        algorithms::shr(&mut self.limbs, n);
    }
    pub fn convert_montgomery(&self, n: &UBigInt) -> UBigInt {
        let r_limbs = n.limbs.len() as u32;
        let mut result = UBigInt::new(r_limbs*LIMB_BITS*2);
        result.set(self);
        montgomery::to_montgomery_form(&mut result.limbs, r_limbs, &n.limbs);
        result
    }
    pub fn reduce_montgomery(&mut self, n: &UBigInt) {
        let r_limbs = n.limbs.len() as u32;
        montgomery::from_montgomery_form(&mut self.limbs, r_limbs, &n.limbs);
    }
    pub fn multiply_montgomery(&self, other: &UBigInt, n: &UBigInt) -> UBigInt {
        let mut result = UBigInt::new(self.bits);
        result.mul_add(self, other);
        result.reduce_montgomery(n);
        result
    }
}

impl PartialEq for UBigInt {
    fn eq(&self, other: &UBigInt) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for UBigInt {}

impl Ord for UBigInt {
    fn cmp(&self, other: &UBigInt) -> Ordering {
        algorithms::compare(&self.limbs, &other.limbs)
    }
}

impl PartialOrd for UBigInt {
    fn partial_cmp(&self, other: &UBigInt) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for UBigInt {
    fn drop(&mut self) {
        // May need to prevent this from being optimized away
        self.limbs.fill_copy(0);
    }
}
