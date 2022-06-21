use std::io;
use std::path::PathBuf;

use clap::Parser;
use env_logger::Env;
use inferno::collapse::xctrace::Folder;
use inferno::collapse::{Collapse, DEFAULT_NTHREADS};

#[derive(Debug, Parser)]
#[clap(name = "inferno-collapse-xctrace", about)]
struct Opt {
    // ************* //
    // *** FLAGS *** //
    // ************* //
    /// Silence all log output
    #[clap(short = 'q', long = "quiet")]
    quiet: bool,

    /// Verbose logging mode (-v, -vv, -vvv)
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    verbose: usize,

    // *************** //
    // *** OPTIONS *** //
    // *************** //
    /// Number of threads to use
    // #[clap(
    //     short = 'n',
    //     long = "nthreads",
    //     default_value = 1,
    //     value_name = "UINT"
    // )]
    // nthreads: usize,

    // ************ //
    // *** ARGS *** //
    // ************ //
    #[clap(value_name = "PATH")]
    /// Perf script output file, or STDIN if not specified
    infile: Option<PathBuf>,
}

// impl Opt {
//     // fn into_parts(self) -> (Option<PathBuf>, Options) {
//     //     let mut options = Options::default();
//     //     options.include_pid = self.pid;
//     //     options.include_tid = self.tid;
//     //     options.include_addrs = self.addrs;
//     //     options.annotate_jit = self.jit || self.all;
//     //     options.annotate_kernel = self.kernel || self.all;
//     //     options.event_filter = self.event_filter;
//     //     options.nthreads = self.nthreads;
//     //     options.skip_after = self.skip_after;
//     //     (self.infile, options)
//     // }
// }

fn main() -> io::Result<()> {
    let opt = Opt::parse();

    // Initialize logger
    if !opt.quiet {
        env_logger::Builder::from_env(Env::default().default_filter_or(match opt.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }))
        .format_timestamp(None)
        .init();
    }

    Folder::default().collapse_file_to_stdout(opt.infile.as_ref())
}
