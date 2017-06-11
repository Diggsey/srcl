
pub struct UBigInt {
    limbs: Box<[Limb]>,
    bits: u32
}

impl UBigInt {
    pub fn new(bits: u32) -> Self {
        assert!(bits > 0);
        UBigInt {
            limbs: vec![0; (bits+LIMB_BITS-1)/LIMB_BITS].into_boxed_slice(),
            bits: bits
        }
    }
    pub fn set_u32(&mut self, value: u32) {
        self.limbs[1..].fill_copy(0);
        self.limbs[0] = value;
    }
    pub fn mul_acc(&mut self, a: &UBigInt, b: &UBigInt) {
        let (x, y) = if b.bits < c.bits {
            (b, c)
        } else {
            (c, b)
        };

        if x.limbs.len() < 4 {

        }
    }
}

impl Drop for UBigInt {
    fn drop(&mut self) {
        // May need to prevent this from being optimized away
        self.limbs.fill_copy(0);
    }
}
