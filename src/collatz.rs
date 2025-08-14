use crate::bigint;

#[derive(Clone)]
pub struct ProductSum {
    product: u32,
    sum: u32,
}

pub fn precompute_constants(search_steps: u64) -> Box<[ProductSum]> {
    let size = 1 << search_steps;
    let mut constants = vec![ProductSum{product: 1, sum: 0}; size].into_boxed_slice();
    let search_steps = match search_steps {
        0 => return Box::new([ProductSum{product: 1, sum: 0}, ProductSum{product: 3, sum: 1}]),
        steps if steps < 21 => steps,
        _ => 20,
    };

    let mut i = 1;
    for constant in constants[1..].iter_mut().step_by(2) {
        let mut search = i as u64;
        for _ in 0..search_steps {
            // if search > (u64::MAX - 1) / 3 { panic!("overflow!!!"); }
            match search % 2 == 1 {
                true => {
                    search = (search * 3 + 1) / 2;
                    constant.product *= 3;
                },
                false => search /= 2,
            }
        }

        let sum = search * (size as u64) - i as u64 * constant.product as u64;
        // if sum > u32::MAX as u64 { panic!("overflow!!!"); }
        constant.sum = sum as u32;
        i += 2;
    }

    constants
}

pub fn precompute_mod_skip(search_steps: u64) -> Box<[u8]> {
    let size = 1 << search_steps;
    let mut mod_skip = vec![0; size / 4].into_boxed_slice();

    // 0 mod 2, 1 mod 4, 3 mod 16, 11 mod 32, 23 mod 32, 7 mod 128, 15 mod 128, 59 mod 128
    // 39 mod 256, 79 mod 256, 95 mod 256, 123 mod 256, 175 mod 256, 199 mod 256, 219 mod 256
    let small_mod_skip = match search_steps {
        ..3 => { return Box::new([4]); },
        ..20 => Box::new([4]),
        ..30 => precompute_mod_skip(18),
        _ => precompute_mod_skip(27),
    };

    let mut i = 3;
    let mut low_search = 3;
    while i < size {
        let mut j = 0;
        let mut search = i;
        let mut lowest_search = i;
        while j < search_steps & !3 {
            const PRODUCTS: [usize; 16] = [1, 9, 9, 9, 3, 3, 9, 27, 3, 27, 3, 27, 9, 9, 27, 81];
            const SUMS: [usize; 16] = [0, 7, 14, 5, 4, 1, 10, 19, 8, 29, 2, 23, 20, 11, 38, 65];
            const LOW_PRODUCTS: [usize; 16] = [1, 9, 6, 9, 3, 3, 8, 16, 2, 12, 3, 16, 4, 6, 8, 16];
            const LOW_SUMS: [usize; 16] = [0, 7, 4, 5, 4, 1, 0, 0, 0, 4, 2, 0, 0, 2, 0, 0];

            // if search > (usize::MAX - 65) / 81 { panic!("overflow!!!"); }
            let index = search % 16;
            lowest_search = search * LOW_PRODUCTS[index] + LOW_SUMS[index];
            search = search * PRODUCTS[index] + SUMS[index];

            search /= 16;
            lowest_search /= 16;
            if lowest_search < i { break; }
            j += 4;
        }

        if lowest_search >= i {
            while j < search_steps {
                // if search > (usize::MAX - 1) / 3 { panic!("overflow!!!"); }
                search = match search % 2 {
                    0 => search / 2,
                    _ => (search * 3 + 1) / 2,
                };
                lowest_search = search;
                if lowest_search < i { break; }
                j += 1;
            }
        }

        let (target, is_target) = match i < low_search + (u8::MAX & !3) as usize {
            true => (i, lowest_search >= i),
            false => (low_search + (u8::MAX & !3) as usize, true)
        };
        
        if is_target {
            let mut k = low_search;
            for skip in mod_skip[low_search / 4..target / 4].iter_mut() {
                *skip = (target - k) as u8;
                k += 4;
            }
            low_search = target;
        }

        i += small_mod_skip[(i / 4) & (small_mod_skip.len() - 1)] as usize;
    }

    mod_skip[size / 4 - 1] = mod_skip[0] + 4;
    mod_skip
}

pub fn compute_range(search_start: u64, search_end: u64, precomputed_constants: std::sync::Arc<Box<[ProductSum]>>, mod_skip: std::sync::Arc<Box<[u8]>>) -> u64 {    
    let constants_mask = precomputed_constants.len() - 1;
    let mod_skip_mask = mod_skip.len() - 1;
    let mut trailing_search = search_start | 3;
    while trailing_search <= search_end {
        let mut current_search = bigint::BigInt{low: trailing_search, high: 0};
        while current_search.low >= trailing_search || current_search.high > 0 {
            let index = current_search.low as usize & constants_mask;
            let prod: u64 = precomputed_constants[index].product.into();
            let sum: u64 = precomputed_constants[index].sum.into();

            current_search = match current_search.checked_mul(prod) {
                Some(result) => result,
                None => { return trailing_search - 1 }
            };
            current_search = match current_search.checked_add(sum) {
                Some(result) => result,
                None => { return trailing_search - 1 }
            };
            current_search = current_search.remove_trailing_zeros();
        }

        const MOD_SKIP_3: [u8; 9] = [0, 0, 1, 0, 1, 1, 0, 0, 1];
        let temp = trailing_search;
        loop {
            trailing_search = trailing_search.wrapping_add(mod_skip[(trailing_search / 4) as usize & mod_skip_mask] as u64);
            if MOD_SKIP_3[(trailing_search % 9) as usize] != 0 { continue }
            break
        }

        if trailing_search < temp { return temp }
    }

    search_end
}