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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    use text_io::read;

    let mut stream = if let Some(path) = args.input_file {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let stream = reader.bytes().map(Result::unwrap);
        Box::new(stream) as Box<dyn Iterator<Item = u8>>
    } else {
        let stream = io::stdin().bytes().map(Result::unwrap);
        Box::new(stream) as Box<dyn Iterator<Item = u8>>
    };

    let bin_capacity: u32 = read!("{}", &mut stream);
    let mut weights = Vec::new();
    while let nonzero @ 1.. = read!("{}", &mut stream) {
        weights.push(nonzero)
    }

    let mutex = sync::Arc::new(sync::Mutex::new(None));
    let other_mutex = mutex.clone();
    let (tx, rx) = sync::mpsc::channel();

    let timeout_thread = thread::spawn(move || {
        if let Some(timeout) = args.timeout {
            if rx.recv_timeout(timeout.into()).is_err() {
                println!("c Timed out after {timeout}");
            }
        } else {
            // wait for the main thread to finish computation
            let _ = rx.recv();
        }

        let best_fit: Option<Vec<Vec<_>>> = other_mutex.lock().unwrap().take();
        match best_fit {
            Some(best_fit) => {
                println!("s SAT");

                if args.values {
                    for row in best_fit {
                        println!(
                            "v {}",
                            row.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                    }
                }
            }
            None => {
                println!("s UNSAT");
            }
        }

        process::exit(0);
    });

    let mut max_bins = weights.len();

    loop {
        println!("c Fitting to {} bins", max_bins);
        // find a fit
        let current_fit: Option<Vec<Vec<_>>> =
            fitter::Fitter::new(weights.clone(), vec![bin_capacity; max_bins])
                .fit()
                .map(|fit| fit.into_iter().filter(|row| !row.is_empty()).collect());

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
