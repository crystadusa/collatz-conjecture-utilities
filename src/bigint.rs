// As of now, BigInts are 128 bit integers
pub struct BigInt {
    pub low: u64,
    pub high: u64,
}

// Key operations optimized for iterating the collatz function
impl BigInt {
    pub const fn checked_add(self, rhs: u32) -> Option<Self> {
        let (low, is_overflow) = self.low.overflowing_add(rhs as u64);
        let (high, is_overflow) = match is_overflow {
            false => (self.high, false),
            true => self.high.overflowing_add(1)
        };

        if is_overflow { return None; }
        Some(BigInt{low, high})
    }

    pub const fn checked_mul(self, rhs: u32) -> Option<Self> {
        let rhs = rhs as u64;
        let (mut high, is_overflow) = self.high.overflowing_mul(rhs);
        if is_overflow { return None }
        
        let (low, mut is_overflow) = self.low.overflowing_mul(rhs);
        if is_overflow {
            let u1 = self.low & u32::MAX as u64;
            let k = u1 * rhs >> 32;
            let t = (self.low >> 32) * rhs + k;
            let overflow = t >> 32;
            
            (high, is_overflow) = high.overflowing_add(overflow);
            if is_overflow { return None }
        };

        Some(BigInt{low, high})
    }
}

// Operator overloading for BigInt
impl std::ops::ShrAssign<u32> for BigInt { 
    // Only works when rhs is greater than 0
    fn shr_assign(&mut self, rhs: u32) {
        let (mut low, mut high) = (self.low, self.high);
        if low == 0 {
            low = high;
            high = 0;
        }

        low >>= rhs;
        self.low = low | (high & ((1u64 << rhs) - 1)).wrapping_shl(64 - rhs);
        self.high = high >> rhs;
    }
}

// Unused algorithm to multiply two u64s into a low and high u64
const fn _bigint_mult(mut op1: u64, mut op2: u64) -> (u64, u64) {
    let u1 = op1 & u32::MAX as u64;
    let v1 = op2 & u32::MAX as u64;

    let t = u1 * v1;
    let w3= t & u32::MAX as u64;
    let k = t >> 32;

    op1 >>= 32;
    let t = op1 * v1 + k;
    let k = t & u32::MAX as u64;
    let w1 = t >> 32;

    op2 >>= 32;
    let t = u1 * op2 + k;
    let k = t >> 32;

    let high = op1 * op2 + w1 + k;
    let low = (t << 32) + w3;
    (high, low)
}