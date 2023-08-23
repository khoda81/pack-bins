use clap::Parser;
use std::io::Read;
use std::{fs, io, path, process, sync, thread};

/// A backtracking solution to bin packing problem
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to parse (uses stdin by default)
    #[arg(short, long)]
    input_file: Option<path::PathBuf>,

    /// Timeout for the computation
    #[arg(short, long)]
    timeout: Option<humantime::Duration>,

    /// Show the values
    #[arg(short, long)]
    values: bool,

    /// Try to minimize the number of bins to use
    #[arg(short, long)]
    minimize_bins: bool,
}

fn parse_input(stream: impl Read) -> anyhow::Result<(u32, Vec<u32>)> {
    use text_io::try_read;

    let mut stream = stream.bytes().map(Result::unwrap);
    let bin_capacity = try_read!("{}", &mut stream)?;
    let mut weights = Vec::new();
    while let nonzero @ 1.. = try_read!("{}", &mut stream)? {
        weights.push(nonzero)
    }

    Ok((bin_capacity, weights))
}

fn print_values(best_fit: Vec<fitter::Bin<u32>>) {
    best_fit.into_iter().for_each(|row| {
        let row = row
            .items()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        println!("v {}", row);
    });
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let stream: Box<dyn Read> = if let Some(path) = args.input_file {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        Box::new(reader)
    } else {
        Box::new(io::stdin())
    };

    let (bin_capacity, weights) = parse_input(stream)?;

    let mutex = sync::Arc::new(sync::Mutex::new(None));
    let other_mutex = mutex.clone();
    let (tx, rx) = sync::mpsc::channel();

    let timeout_thread = thread::spawn(move || {
        let mut timed_out = false;

        if let Some(timeout) = args.timeout {
            if rx.recv_timeout(timeout.into()).is_err() {
                println!("c Timed out after {timeout}");
                timed_out = true;
            }
        } else {
            // wait for the main thread to finish computation
            let _ = rx.recv();
        }

        let best_fit = other_mutex.lock().unwrap().take();
        match best_fit {
            None if timed_out => println!("s UNKNOWN"),
            None => println!("s UNSAT"),

            Some(best_fit) => {
                println!("s SAT");

                if args.values {
                    print_values(best_fit);
                }
            }
        }

        process::exit(0);
    });

    let mut max_bins = weights.len();

    loop {
        println!("c Trying to fit in {max_bins} bins");

        // find a fit
        let current_fit = fitter::Fitter::new(weights.clone(), vec![bin_capacity; max_bins])
            .fit()
            .map(|fit| {
                fit.into_iter()
                    .filter(|row| !row.is_empty())
                    .collect::<Vec<_>>()
            });

        if let Some(current_fit) = current_fit {
            max_bins = current_fit.len().saturating_sub(1);

            // send the solution through mutex
            let _ = mutex.lock().unwrap().insert(current_fit);
        } else {
            break;
        }

        if max_bins == 0 || !args.minimize_bins {
            break;
        }
    }

    let _ = tx.send(());
    let _ = timeout_thread.join();

    anyhow::Ok(())
}
