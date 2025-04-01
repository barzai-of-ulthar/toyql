use clap::Parser;

/// The ToyQL query engine.
///
/// Processes queries, either from the command line or from a file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// Files containing query text
    #[arg(short = 'f', long = "file")]
    query_files: Vec<String>,

    /// Literal query text
    queries: Vec<String>,
}

fn run_from_args(args: Args) {
    for file in args.query_files {
        println!("I would parse the external file {}", file);
    }
    for query in args.queries {
        println!("I would execute the query {}", query);
    }
}

/// The "real" main entry point of our binary target, relocated to a lib so that
/// rust-test can reach it.
pub fn _main() {
    let args = Args::parse();

    run_from_args(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        run_from_args(Args{query_files:vec!(), queries:vec!()});
    }    
}
