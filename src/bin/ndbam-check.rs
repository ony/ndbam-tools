extern crate structopt;

use std::path::PathBuf;
use ndbam::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    /// Location of database
    #[structopt(short = "l", long = "location", default_value = "/var/db/paludis/repositories/installed")]
    location: PathBuf,
}

fn main() {
    let opts = Opts::from_args();
    println!("Hello, Exherbo!");
    let reg = NDBAM::new(&opts.location);
    reg.all_packages()
        .map(|iter|
             iter.for_each(|pkg| {
                 println!("{}:{}", pkg.full_name(), pkg.slot().unwrap_or("0"));
                 if let Ok(summary) = pkg.read_key("SUMMARY") {
                     if !summary.trim_end().is_empty() { println!("  Summary: {}", summary.trim_end()) }
                 }
             })
        );
}
