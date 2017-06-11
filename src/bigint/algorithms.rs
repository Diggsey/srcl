use std::iter;

use utils::slice_ext::SliceExt;

type Limb = u32;
type DLimb = u64;
const LIMB_BITS: u32 = 32;
const LIMB_MASK: DLimb = (1 << LIMB_BITS)-1;

// These algorithms are borrowed from the `num::bigint` crate, with modifications to
// make them constant time (modulo compiler optimizations)

fn split_dlimb(a: DLimb) -> (Limb, Limb) {
    ((a & LIMB_MASK) as Limb, (a >> LIMB_BITS) as Limb)
}

fn mul_add_with_carry(acc: &mut Limb, b: Limb, c: Limb, carry: &mut Limb) {
    let (lo, hi) = split_dlimb(
        (*acc as DLimb)
        + (b as DLimb)*(c as DLimb)
        + (*carry as DLimb)
    );
    *acc = lo;
    *carry = hi;
}

fn add_with_carry(acc: &mut Limb, b: Limb, carry: &mut Limb) {
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

fn limb_mul_add(acc: &mut [Limb], b: &[Limb], c: Limb) {
    let mut b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut carry = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        mul_add_with_carry(ai, bi, c, &mut carry);
    }

    assert!(carry == 0);
}

fn add(acc: &mut [Limb], b: &[Limb]) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut carry = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        add_with_carry(ai, bi, &mut carry);
    }

    assert!(carry == 0);
}

fn sub(acc: &mut [Limb], b: &[Limb]) {
    let b_iter = b.iter().cloned().chain(iter::repeat(0));
    let mut borrow = 0;

    for (ai, bi) in acc.iter_mut().zip(b_iter) {
        sub_with_borrow(ai, bi, &mut borrow);
    }

    assert!(borrow == 0);
}

fn mul_add(acc: &mut [Limb], b: &[Limb], c: &[Limb]) {
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
}
