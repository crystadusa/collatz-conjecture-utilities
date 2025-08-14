mod bigint;
mod collatz;

fn main() {
    let mut search_start = 2;
    let mut search_cap = 0;
    let mut thread_count = 1;
    let mut constants_size = 16;
    let mut mod_skip_size = 24;
    let mut numbers_entered = 0;        
    for arg in std::env::args().skip(1) {
        let arg_type = arg.trim_end_matches(char::is_numeric);
        match arg_type {
            "-h" | "-help" => {
                println!("collatz <End> <Options>\n\
                    collatz <Start> <End> <Options>\n\
                    Options:\n  \
                    -c -const-steps<Precomputed stpes for a multi step table>\n  \
                    -m -mod-steps<Precomputed steps in a modular skip table>\n  \
                    -t -threads<Number of Threads>");
                return;
            }
            "-c" | "-const-steps" => {
                match arg[arg_type.len()..].parse::<u64>() {
                    Ok(arg) => constants_size = arg,
                    Err(_) => {
                        println!("Constants table steps is not a number!");
                        return;
                    }
                }
            }
            "-m" | "-mod-steps" => {
                match arg[arg_type.len()..].parse::<u64>() {
                    Ok(arg) => mod_skip_size = arg,
                    Err(_) => {
                        println!("Modular table steps is not a number!");
                        return;
                    }
                }
            }
            "-t" | "-threads" => {
                match arg[arg_type.len()..].parse::<u64>() {
                    Ok(arg) => thread_count = arg,
                    Err(_) => {
                        println!("Thread count is not a number!");
                        return;
                    }
                }
            }
            _ => match arg.parse::<u64>() {
                Ok(arg) => {
                    match numbers_entered {
                        0 => search_cap = arg,
                        1 => {
                            search_start = search_cap;
                            search_cap = arg;
                        }
                        _ => {
                            println!("Unrecognised numerical argument!");
                            return;
                        }
                    }
                    numbers_entered += 1;
                }
                Err(_) => {
                    println!("Unrecognised argument {arg}!");
                    return;
                }
            }
        }
    }

    if numbers_entered == 0 {
        println!("Please provide a maximum search depth!\nType collatz -h for more information.");
        return;
    }
    if search_start < 2 {
        println!("Search start must be at least 2!");
        return;
    }
    if thread_count == 0 {
        println!("Thread count must be at least 1!");
        return;
    }

    let precomputation_start = std::time::Instant::now();
    let precomputed_constants = std::sync::Arc::new(collatz::precompute_constants(constants_size));
    let mod_skip = std::sync::Arc::new(collatz::precompute_mod_skip(mod_skip_size));
    let precomputation_time = precomputation_start.elapsed().as_millis();
    println!("The precomputation took {} seconds to complete.", precomputation_time as f32 / 1000.0);

    let start_time = std::time::Instant::now();
    let search_size = search_cap - search_start + 1;
    let thread_size = (search_size - 1) / thread_count + 1;
    
    let mut trailing_search = search_start;
    if thread_count == 1 {
        trailing_search = collatz::compute_range(search_start, search_cap, precomputed_constants, mod_skip);
        if trailing_search < search_cap {
            println!("Overflow occurred at {} in the search!", trailing_search);
        }
    } else {
        let mut threads = vec![];
        for i in 0..thread_count {
            let thread_start = search_start + thread_size * i;
            let mut thread_end = thread_start + thread_size - 1;
            if thread_end > search_cap { thread_end = search_cap; }

            let constants = precomputed_constants.clone();
            let mod_skip = mod_skip.clone();
            threads.push(std::thread::spawn(move || {
                collatz::compute_range(thread_start, thread_end, constants, mod_skip)
            }));
        }

        let mut i = 0;
        for thread in threads {
            let mut thread_end = search_start + thread_size * (i + 1) - 1;
            if thread_end > search_cap { thread_end = search_cap; }

            trailing_search = match thread.join() {
                Ok(result) => result,
                Err(_) => panic!("Failed to wait on thread!")
            };
            if trailing_search < thread_end {
                println!("Overflow occurred at {} in the search!", trailing_search);
            }
            i += 1;
        }
    }

    let end_time = start_time.elapsed().as_millis();
    println!("The search took {} seconds to complete.", end_time as f32 / 1000.0);
    println!("Numbers {search_start} through {trailing_search} go below themselves in the collatz conjecture.");
}