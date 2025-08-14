pub struct BigInt {
    pub low: u64,
    pub high: u64,
}

impl BigInt {
    pub const fn checked_add(self, rhs: u64) -> Option<Self> {
        let (low, overflow) = self.low.overflowing_add(rhs);
        let high = match overflow {
            false => self.high,
            true => match self.high.checked_add(1) {
                Some(result) => result,
                None => return None,
            }
        };
        Some(BigInt{low, high,})
    }

    pub const fn checked_mul(self, rhs: u64) -> Option<Self> {
        let mut high = match self.high.checked_mul(rhs) {
            Some(result) => result,
            None => return None,
        };

        let (low, overflow) = self.low.overflowing_mul(rhs);
        if overflow {
            let u1 = self.low & u32::MAX as u64;
            let k = u1 * rhs >> 32;
            let t = (self.low >> 32) * rhs + k;
            let overflow = t >> 32;
            
            high = match high.checked_add(overflow) {
                Some(result) => result,
                None => return None,
            }
        };

        Some(BigInt{low, high,})
    }

    pub const fn remove_trailing_zeros(self) -> Self {
        let (mut low, mut high) = (self.low, self.high);
        if low == 0 {
            low = high;
            high = 0;
        }

        let shift = low.trailing_zeros();
        low >>= shift;
        low |= (high & ((1u64 << shift) - 1)).wrapping_shl(64 - shift);
        high >>= shift;

        BigInt{low, high}
    }
}

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