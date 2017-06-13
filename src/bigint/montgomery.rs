use std::cmp::Ordering;

use super::algorithms::{self, Limb, LIMB_BITS};

pub fn to_montgomery_form(a: &mut [Limb], r_limbs: u32, n: &[Limb]) {
    algorithms::shl(a, r_limbs*LIMB_BITS);
    algorithms::pure_mod(a, n);
}

pub fn from_montgomery_form(a: &mut [Limb], r_limbs: u32, n: &[Limb]) {
    let n_low = n[0];
    let n_low_prime = algorithms::compute_limb_n_prime(n_low);

    for i in 0..(r_limbs as usize) {
        let mut carry = 0;
        let m = a[i].wrapping_mul(n_low_prime);
        for j in 0..n.len() {
            algorithms::mul_add_with_carry(&mut a[i+j], m, n[j], &mut carry);
        }
        for j in n.len()..(a.len() - i) {
            algorithms::add_with_carry(&mut a[i+j], 0, &mut carry);
        }
    }
    algorithms::shr(a, r_limbs*LIMB_BITS);

    let factor = if algorithms::compare(a, n) == Ordering::Less { 0 } else { 1 };
    algorithms::limb_mul_sub(a, n, factor);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_from() {
        let (mut a, n) = ([2, 1, 0, 0, 0, 0, 0, 0], [5, 7]);
        println!("{:?}", a);
        to_montgomery_form(&mut a, 1, &n);
        println!("{:?}", a);
        from_montgomery_form(&mut a, 1, &n);
        println!("{:?}", a);
        assert!(a == [2, 1, 0, 0, 0, 0, 0, 0]);
    }
}
