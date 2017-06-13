use std::iter;
use std::cmp::{self, Ordering};
use std::mem;

use utils::slice_ext::SliceExt;

pub type Limb = u32;
pub type DLimb = u64;
pub const LIMB_BITS: u32 = 32;
pub const LIMB_MASK: DLimb = (1 << LIMB_BITS)-1;

// These algorithms are borrowed from the `num::bigint` crate, with modifications to
// make them constant time (modulo compiler optimizations)

fn split_dlimb(a: DLimb) -> (Limb, Limb) {
    ((a & LIMB_MASK) as Limb, (a >> LIMB_BITS) as Limb)
}

pub fn mul_add_with_carry(acc: &mut Limb, b: Limb, c: Limb, carry: &mut Limb) {
    let (lo, hi) = split_dlimb(
        (*acc as DLimb)
        + (b as DLimb)*(c as DLimb)
        + (*carry as DLimb)
    );
    *acc = lo;
    *carry = hi;
}

fn mul_sub_with_borrow(acc: &mut Limb, b: Limb, c: Limb, borrow: &mut Limb) {
    let (lo, hi) = split_dlimb(
        (*acc as DLimb)
        .wrapping_sub((b as DLimb)*(c as DLimb))
        .wrapping_sub(*borrow as DLimb)
    );
    *acc = lo;
    *borrow = hi.wrapping_neg();
}

pub fn add_with_carry(acc: &mut Limb, b: Limb, carry: &mut Limb) {
    let (lo, hi) = split_dlimb(
        (*acc as DLimb)
        + (b as DLimb)
        + (*carry as DLimb)
    );
    *acc = lo;
    *carry = hi;
}

fn sub_with_borrow(acc: &mut Limb, b: Limb, borrow: &mut Limb) {
    let (lo, hi) = split_dlimb(
        (*acc as DLimb)
        .wrapping_sub(b as DLimb)
        .wrapping_sub(*borrow as DLimb)
    );
    *acc = lo;
    *borrow = hi.wrapping_neg();
}

fn shl_with_carry(acc: &mut Limb, b: u32, carry: &mut Limb) {
    let (lo, hi) = split_dlimb(
        ((*acc as DLimb) << b) | (*carry as DLimb)
    );
    *acc = lo;
    *carry = hi;
}

fn shr_with_carry(acc: &mut Limb, b: u32, carry: &mut Limb) {
    let (lo, hi) = split_dlimb(
        ((*acc as DLimb) | ((*carry as DLimb) << LIMB_BITS)) << (LIMB_BITS - b)
    );
    *acc = hi;
    *carry = lo;
}

fn limb_mul_add(acc: &mut [Limb], b: &[Limb], c: Limb) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut carry = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        mul_add_with_carry(ai, bi, c, &mut carry);
    }

    assert!(carry == 0);
}

pub fn limb_mul_sub(acc: &mut [Limb], b: &[Limb], c: Limb) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut borrow = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        mul_sub_with_borrow(ai, bi, c, &mut borrow);
    }

    assert!(borrow == 0);
}

pub fn add(acc: &mut [Limb], b: &[Limb]) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut carry = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        add_with_carry(ai, bi, &mut carry);
    }

    assert!(carry == 0);
}

pub fn sub(acc: &mut [Limb], b: &[Limb]) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut borrow = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        sub_with_borrow(ai, bi, &mut borrow);
    }

    assert!(borrow == 0);
}

pub fn shl(acc: &mut [Limb], b: u32) {
    let mut carry = 0;
    let (bl, bb) = ((b / LIMB_BITS) as usize, b % LIMB_BITS);

    for i in (bl..acc.len()).rev() {
        acc[i] = acc[i-bl];
    }
    for i in (0..bl).rev() {
        acc[i] = 0;
    }
    for ai in acc {
        shl_with_carry(ai, bb, &mut carry);
    }

    assert!(carry == 0);
}

pub fn shr(acc: &mut [Limb], b: u32) {
    let mut carry = 0;
    let (bl, bb) = ((b / LIMB_BITS) as usize, b % LIMB_BITS);

    for i in 0..(acc.len()-bl) {
        acc[i] = acc[i+bl];
    }
    for i in acc.len()-bl..acc.len() {
        acc[i] = 0;
    }
    for ai in acc.iter_mut().rev() {
        shr_with_carry(ai, bb, &mut carry);
    }

    assert!(carry == 0);
}

pub fn mul_add(acc: &mut [Limb], b: &[Limb], c: &[Limb]) {
    let (x, y) = if b.len() < c.len() {
        (b, c)
    } else {
        (c, b)
    };

    // Karatsuba multiplication is slower than long multiplication for small x and y:
    //
    if x.len() <= 4 {
        for (i, &xi) in x.iter().enumerate() {
            limb_mul_add(&mut acc[i..], y, xi);
        }
    } else {
        // Karatsuba multiplication

        // x = x0 + B*x1
        // y = y0 + B*y1

        // x*y	= (x0 + B*x1)*(y0 + B*y1)
        //     = x0*y0 + B*x0*y1 + B*x1*y0 + B^2*x1*y1
        //     = z0 + B*z1 + B^2*z2

        // z0	= x0*y0
        // z2	= x1*y1
        // z1	= x0*y1 + x1*y0
        //     = x0*y1 + x1*y0 + x0*y0 + x1*y1 - z0 - z2
        //     = (x0 + x1)*(y0 + y1) - z0 - z2

        // r0 = x0 + x1
        // r1 = y0 + y1
        // acc[B..] += r0*r1

        // r01 = x1*y1
        // acc[B^2..] += r01
        // acc[B..] -= r01

        // r01 = x0*y0
        // acc += r01
        // acc[B..] -= r01


        // Split the smaller side in half, so we can take the other branch sooner
        let b = x.len() / 2;
        let (x0, x1) = x.split_at(b);
        let (y0, y1) = y.split_at(b);

        // We reuse the same space for r01, r0 and r1, so the size constraints are:
        //
        // 1) r0 = x0 + x1
        // => len(r0) > max(len(x0), len(x1))
        // => len(r0) > len(x1)
        //
        // 2) r1 = y0 + y1
        // => len(r1) > max(len(y0), len(y1))
        // => len(r1) > len(y1)
        //
        // 3) r01 = x1*y1
        // => len(r0) + len(r1) >= len(x1) + len(y1)
        //
        // 4) r01 = x0*y0
        // => len(r0) + len(r1) >= len(x0) + len(y0)
        //
        // Combined, this gives:
        // => len(r0) > len(x1) && len(r1) > len(y1)

        let r0_len = x1.len() + 1;
        let r1_len = y1.len() + 1;
        let mut r01_box = vec![0; r0_len + r1_len].into_boxed_slice();
        let r01 = &mut r01_box[..];

        {
            let (r0, r1) = r01.split_at_mut(r0_len);
            // r0 = x0 + x1
            r0[..x0.len()].copy_from_slice(x0);
            add(r0, x1);
            // r1 = y0 + y1
            r1[..y0.len()].copy_from_slice(y0);
            add(r1, y1);
            // acc[B..] += r0*r1
            mul_add(&mut acc[b..], r0, r1);
        }

        // r01 = x1*y1
        r01.fill_copy(0);
        mul_add(r01, x1, y1);
        // acc[B^2..] += r01
        add(&mut acc[b*2..], r01);
        // acc[B..] -= r01
        sub(&mut acc[b..],   r01);

        // r01 = x0*y0
        r01.fill_copy(0);
        mul_add(r01, x0, y0);
        // acc += r01
        add(&mut acc[..],    r01);
        // acc[B..] -= r01
        sub(&mut acc[b..],   r01);

        // Don't leave anything around
        r01.fill_copy(0);
    }
}

// Same as `mul_add` except we're subtracting, so we alter
// the order of operations slightly to avoid going negative
pub fn mul_sub(acc: &mut [Limb], b: &[Limb], c: &[Limb]) {
    let (x, y) = if b.len() < c.len() {
        (b, c)
    } else {
        (c, b)
    };

    // Karatsuba multiplication is slower than long multiplication for small x and y:
    //
    if x.len() <= 4 {
        for (i, &xi) in x.iter().enumerate() {
            limb_mul_sub(&mut acc[i..], y, xi);
        }
    } else {
        // Karatsuba multiplication

        // x = x0 + B*x1
        // y = y0 + B*y1

        // x*y	= (x0 + B*x1)*(y0 + B*y1)
        //     = x0*y0 + B*x0*y1 + B*x1*y0 + B^2*x1*y1
        //     = z0 + B*z1 + B^2*z2

        // z0	= x0*y0
        // z2	= x1*y1
        // z1	= x0*y1 + x1*y0
        //     = x0*y1 + x1*y0 + x0*y0 + x1*y1 - z0 - z2
        //     = (x0 + x1)*(y0 + y1) - z0 - z2

        // r01 = x0*y0
        // acc -= r01
        // acc[B..] += r01

        // r01 = x1*y1
        // acc[B^2..] -= r01
        // acc[B..] += r01

        // r0 = x0 + x1
        // r1 = y0 + y1
        // acc[B..] -= r0*r1


        // Split the smaller side in half, so we can take the other branch sooner
        let b = x.len() / 2;
        let (x0, x1) = x.split_at(b);
        let (y0, y1) = y.split_at(b);

        // We reuse the same space for r01, r0 and r1, so the size constraints are:
        //
        // 1) r0 = x0 + x1
        // => len(r0) > max(len(x0), len(x1))
        // => len(r0) > len(x1)
        //
        // 2) r1 = y0 + y1
        // => len(r1) > max(len(y0), len(y1))
        // => len(r1) > len(y1)
        //
        // 3) r01 = x1*y1
        // => len(r0) + len(r1) >= len(x1) + len(y1)
        //
        // 4) r01 = x0*y0
        // => len(r0) + len(r1) >= len(x0) + len(y0)
        //
        // Combined, this gives:
        // => len(r0) > len(x1) && len(r1) > len(y1)

        let r0_len = x1.len() + 1;
        let r1_len = y1.len() + 1;
        let mut r01_box = vec![0; r0_len + r1_len].into_boxed_slice();
        let r01 = &mut r01_box[..];

        // r01 = x0*y0
        mul_add(r01, x0, y0);
        // acc[B..] -= r01
        add(&mut acc[b..],   r01);
        // acc += r01
        sub(&mut acc[..],    r01);
        r01.fill_copy(0);

        // r01 = x1*y1
        mul_add(r01, x1, y1);
        // acc[B..] -= r01
        add(&mut acc[b..],   r01);
        // acc[B^2..] += r01
        sub(&mut acc[b*2..], r01);
        r01.fill_copy(0);

        {
            let (r0, r1) = r01.split_at_mut(r0_len);
            // r0 = x0 + x1
            r0[..x0.len()].copy_from_slice(x0);
            add(r0, x1);
            // r1 = y0 + y1
            r1[..y0.len()].copy_from_slice(y0);
            add(r1, y1);
            // acc[B..] += r0*r1
            mul_sub(&mut acc[b..], r0, r1);
        }

        // Don't leave anything around
        r01.fill_copy(0);
    }
}

fn choose<T>(condition: bool, a: T, b: T) -> T {
    if condition { a } else { b }
}

fn safe_shr(limb: Limb, shift: u32) -> Limb {
    if shift == LIMB_BITS {
        0
    } else {
        limb >> shift
    }
}

pub fn compare(a: &[Limb], b: &[Limb]) -> Ordering {
    let l = cmp::min(a.len(), b.len());
    let mut result = Ordering::Equal;
    for i in 0..l {
        result = match a[i].cmp(&b[i]) {
            Ordering::Equal => result,
            other => other
        };
    }
    for i in l..a.len() {
        result = match a[i].cmp(&0) {
            Ordering::Equal => result,
            other => other
        };
    }
    for i in l..b.len() {
        result = match 0.cmp(&b[i]) {
            Ordering::Equal => result,
            other => other
        };
    }

    result
}

pub fn compare_shifted(a: &[Limb], b: &[Limb], shift: u32) -> Ordering {
    let l = cmp::min(a.len(), b.len());
    let mut result = Ordering::Equal;
    let mut prevb = 0;
    for i in 0..l {
        let bv = b[i] << shift | safe_shr(prevb, LIMB_BITS - shift);
        prevb = b[i];
        result = match a[i].cmp(&bv) {
            Ordering::Equal => result,
            other => other
        };
    }
    for i in l..a.len() {
        let bv = safe_shr(prevb, LIMB_BITS - shift);
        prevb = 0;
        result = match a[i].cmp(&bv) {
            Ordering::Equal => result,
            other => other
        };
    }
    for i in l..b.len() {
        let bv = b[i] << shift | safe_shr(prevb, LIMB_BITS - shift);
        prevb = b[i];
        result = match 0.cmp(&bv) {
            Ordering::Equal => result,
            other => other
        };
    }
    let bv = safe_shr(prevb, LIMB_BITS - shift);
    result = match 0.cmp(&bv) {
        Ordering::Equal => result,
        other => other
    };

    result
}

pub fn div_mod(out: &mut [Limb], a: &mut [Limb], b: &[Limb]) {
    for (i, o) in out.iter_mut().enumerate().rev() {
        *o = 0;
        if i < a.len() {
            let sa = &mut a[i..];
            for bit in (0..32).rev() {
                let v = choose(compare_shifted(sa, b, bit) == Ordering::Less, 0, 1) << bit;
                limb_mul_sub(sa, b, v);
                *o |= v;
            }
        }
    }
}

pub fn pure_mod(a: &mut [Limb], b: &[Limb]) {
    for i in (0..a.len()).rev() {
        let sa = &mut a[i..];
        for bit in (0..32).rev() {
            let v = choose(compare_shifted(sa, b, bit) == Ordering::Less, 0, 1) << bit;
            limb_mul_sub(sa, b, v);
        }
    }
}

fn extended_euclidean(a: u64, b: u64) -> (u64, u64) {
    let (mut s, mut old_s) = (0u64, 1);
    let (mut t, mut old_t) = (1u64, 0);
    let (mut r, mut old_r) = (b, a);

    while r != 0 {
        let quotient = old_r / r;

        mem::swap(&mut old_r, &mut r);
        mem::swap(&mut old_s, &mut s);
        mem::swap(&mut old_t, &mut t);

        r -= quotient*old_r;
        s = s.wrapping_sub(quotient.wrapping_mul(old_s));
        t = t.wrapping_sub(quotient.wrapping_mul(old_t));
    }
    (old_s, old_t)
}

pub fn compute_limb_n_prime(n: Limb) -> Limb {
    (extended_euclidean(1 << LIMB_BITS, n as u64).1 as Limb).wrapping_neg()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_add_with_carry() {
        let (mut a, b, c, mut d) = (1, 2, 3, 4);
        mul_add_with_carry(&mut a, b, c, &mut d);
        assert!(a == 11);
        assert!(d == 0);

        let (mut a, b, c, mut d) = (1, 123456, 789012, 4);
        mul_add_with_carry(&mut a, b, c, &mut d);
        assert!(a == 2918984965);
        assert!(d == 22);
    }

    #[test]
    fn test_mul_add() {
        let (mut a, b, c) = ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], [2, 3, 4, 5, 6], [5, 6, 7, 8, 9]);
        mul_add(&mut a, &b, &c);
        println!("{:?}", a);
        assert!(a == [11, 29, 55, 90, 135, 136, 125, 101, 63, 10]);
    }

    #[test]
    fn test_div_mod() {
        let (mut out, mut a, b) = ([0, 0, 0, 0, 0], [10, 27, 52, 86, 130, 130, 118, 93, 54, 0], [2, 3, 4, 5, 6]);
        div_mod(&mut out, &mut a, &b);
        println!("{:?}", a);
        assert!(out == [5, 6, 7, 8, 9]);
        assert!(a == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_compute_limb_n_prime() {
        for i in 1..1000 {
            let n = i*10418 + 19;
            let n_prime = compute_limb_n_prime(n);
            let nn_prime = n.wrapping_mul(n_prime);
            println!("{}, {}, {}", n, n_prime, nn_prime);
            assert!(nn_prime == LIMB_MASK as Limb);
        }
    }
}
