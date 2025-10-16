use crate::bigint;

#[derive(Clone)]
pub struct ProductSum {
    product: u32,
    sum: u32,
}

// Calculates a table of products and correction sums to simulate "search_steps" iterations of the collatz function
pub fn precompute_constants(search_steps: u64) -> Box<[ProductSum]> {
    // Calculates table size and caps steps at 20 to prevent overflow
    let search_steps = match search_steps {
        0 => return Box::new([ProductSum{product: 1, sum: 0}, ProductSum{product: 3, sum: 1}]),
        steps if steps < 21 => steps,
        _ => 20,
    };

    // Allocates the product and sum table before mutably iterating
    let size = 1 << search_steps;
    let mut constants = vec![ProductSum{product: 1, sum: 0}; size].into_boxed_slice();

    // Parity sequence equivalence https://en.wikipedia.org/wiki/Collatz_conjecture#Optimizations
    // a * 2^k + b = a * product + sum, where product = 3^odd iteration count, and sum = k collatz iterations on b
    let mut i = 0;
    for constant in constants.iter_mut() {
        // Iterates collatz function to group times 3 multiplications into one product
        let mut search = i;
        for _ in 0..search_steps {
            debug_assert!(search <= (u64::MAX - 1) / 3, "Precomputation overflowed!");
            match search % 2 == 1 {
                true => {
                    search = (search * 3 + 1) / 2;
                    constant.product *= 3;
                },
                false => search /= 2,
            }
        }

        // Correction sum from the application of the product on a shifted right argument
        debug_assert!(search <= u32::MAX as u64, "Precomputation overflowed!");
        constant.sum = search as u32;
        i += 1;
    }

    constants
}

// Returns a table of offsets to the next collatz argument that possibly fails to go below itself
pub fn precompute_mod_skip(search_steps: u64) -> Box<[u8]> {
    // Smaller modular skip tables are recursively calculated as an optimization
    let small_mod_skip = match search_steps {
        ..3 => { return Box::new([4]); },
        ..20 => Box::new([4]),
        ..30 => precompute_mod_skip(18),
        _ => precompute_mod_skip(27),
    };

    // It is impossible for small_mod_skip to be empty
    if small_mod_skip.len() == 0 { return Box::new([4]); }

    // Allocates the table of offsets before mutably iterating
    let size = 1 << search_steps;
    let mut mod_skip = vec![0; size / 4].into_boxed_slice();

    let mut i = 3;
    let mut low_search = 3;
    while i < size {
        let mut j = 0;
        let mut search = i;
        let mut lowest_search = i;

        // Four iterations of testing the lowest search at a time
        while j < search_steps & !3 {
            const PRODUCTS: [usize; 16] = [1, 9, 9, 9, 3, 3, 9, 27, 3, 27, 3, 27, 9, 9, 27, 81];
            const SUMS: [usize; 16] = [0, 7, 14, 5, 4, 1, 10, 19, 8, 29, 2, 23, 20, 11, 38, 65];
            const LOW_PRODUCTS: [usize; 16] = [1, 9, 6, 9, 3, 3, 8, 16, 2, 12, 3, 16, 4, 6, 8, 16];
            const LOW_SUMS: [usize; 16] = [0, 7, 4, 5, 4, 1, 0, 0, 0, 4, 2, 0, 0, 2, 0, 0];

            debug_assert!(search <= (usize::MAX - 65) / 81, "Precomputation overflowed!");
            let index = search % 16;
            lowest_search = search * LOW_PRODUCTS[index] + LOW_SUMS[index];
            search = search * PRODUCTS[index] + SUMS[index];

            search /= 16;
            lowest_search /= 16;
            if lowest_search < i { break; }
            j += 4;
        }

        // Remaining iterations are handled one at a time
        if lowest_search >= i {
            while j < search_steps {
                debug_assert!(search <= (usize::MAX - 1) / 3, "Precomputation overflowed!");
                search = match search % 2 {
                    0 => search / 2,
                    _ => (search * 3 + 1) / 2,
                };
                lowest_search = search;
                if lowest_search < i { break; }
                j += 1;
            }
        }

        // Only 3 (mod 4) numbers are iterated, since only these can possibly go below themselves
        // Return table stores u8 values, so the highest increment is 252
        let highest_target = low_search + (u8::MAX & !3) as usize;
        let (target, is_target) = match i < highest_target {
            true => (i, lowest_search >= i),
            false => (highest_target, true)
        };
        
        if is_target {
            // Return table is filled with values to sum with to reach the next target
            let mut k = low_search;
            for skip in mod_skip[low_search / 4..target / 4].iter_mut() {
                *skip = (target - k) as u8;
                k += 4;
            }
            low_search = target;
        }

        // Skips target checks using modular skip table
        i += small_mod_skip[(i / 4) & (small_mod_skip.len() - 1)] as usize;
    }

    // The final value is a special case simpler to handle outside the loop
    mod_skip[size / 4 - 1] = mod_skip[0] + 4;
    mod_skip
}

// Validates the collatz function goes below itself in a given range
pub fn compute_range(search_start: u64, search_end: u64, precomputed_constants: std::sync::Arc<Box<[ProductSum]>>, mod_skip: std::sync::Arc<Box<[u8]>>) -> u64 {
    // The precomputed tables must be filled
    if precomputed_constants.len() == 0 || mod_skip.len() == 0 { return 0 }

    let constants_mask = precomputed_constants.len() - 1;
    let constants_steps = precomputed_constants.len().ilog(2);
    let mod_skip_mask = mod_skip.len() - 1;

    // Rounds up the search start so only 3 (mod 4) numbers are iterated, since only these can possibly go below themselves
    let mut trailing_search = search_start | 3;
    while trailing_search <= search_end {
        let mut current_search = bigint::BigInt{low: trailing_search, high: 0};
        while current_search.low >= trailing_search || current_search.high > 0 {
            let index = current_search.low as usize & constants_mask;
            let prod = precomputed_constants[index].product.into();
            let sum = precomputed_constants[index].sum.into();

            // Parity sequence equivalence https://en.wikipedia.org/wiki/Collatz_conjecture#Optimizations
            // a * 2^k + b >> k = a
            current_search >>= constants_steps;

            // Emulates several collatz iterations at once with precomputed products and sums
            current_search = match current_search.checked_mul(prod) {
                Some(result) => result,
                None => { return trailing_search - 1 }
            };
            current_search = match current_search.checked_add(sum) {
                Some(result) => result,
                None => { return trailing_search - 1 }
            };
        }

        // The precomputed mod power of two table skips search values that go below themselves in a specified number of initial iterations
        // The mod 9 table skips search values that were iterated from a lower search value
        const MOD_SKIP_3: [bool; 9] = [false, false, true, false, true, true, false, false, true];
        let search = trailing_search;
        loop {
            trailing_search = trailing_search.wrapping_add(mod_skip[(trailing_search as usize / 4) & mod_skip_mask].into());
            if MOD_SKIP_3[trailing_search as usize % 9] { continue }
            break
        }

        // "search" is used to handle the special case of overflowing trailing_search
        if trailing_search < search { return search }
    }

    search_end
}