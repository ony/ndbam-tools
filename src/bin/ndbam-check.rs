extern crate structopt;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use ndbam::*;
use ndbam::contents::*;
use crypto_hash::{Algorithm, Hasher};
use structopt::StructOpt;
use colored::*;

#[derive(StructOpt, Debug)]
struct Opts {
    /// Location of database
    #[structopt(short = "l", long = "location", default_value = "/var/db/paludis/repositories/installed")]
    location: PathBuf,

    /// Skip checking package contents
    #[structopt(long = "no-contents")]
    no_contents: bool,

    /// Allow modification time changes
    #[structopt(long = "allow-mtime")]
    allow_mtime: bool,

    /// Skip file integrity checking
    #[structopt(long = "no-integrity")]
    no_integrity: bool,

    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

fn main() {
    let opts = Opts::from_args();
    let reg = NDBAM::new(&opts.location);
    reg.all_packages()
        .map(|iter|
             for pkg in iter {
                 let mut header_shown = false;
                 let mut header = || {
                     if header_shown { return }
                     header_shown = true;
                     println!("{}:{}", pkg.full_name(), pkg.slot().unwrap_or("0"));
                     if let Ok(summary) = pkg.read_key("SUMMARY") {
                         if !summary.trim_end().is_empty() {
                             println!("  # {}: {}", "Summary".bold(), summary.trim_end())
                         }
                     }
                 };
                 if opts.verbose {
                     header()
                 }
                 if !opts.no_contents {
                     check_contents(&opts, &pkg, &mut header)
                 }
             }
        );
}


fn check_contents(opts: &Opts, pkg: &PackageView, header: &mut impl FnMut()) {
    for entry in pkg.contents() {
        if opts.verbose {
            println!("  {:?}", entry);
        }

        let path = entry.path();
        let metadata = match path.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                header();
                if err.kind() == std::io::ErrorKind::NotFound {
                    println!("  X {} Does not exist", path.to_string_lossy().red());
                } else {
                    println!("  X {} {}", path.to_string_lossy().red(), err.to_string());
                }
                continue;
            }
        };

        if !opts.allow_mtime {
            let mtime_changed = match (entry.mtime(), metadata.modified()) {
                (Some(mtime_expect), Ok(mtime)) => epoch_secs(mtime_expect) != epoch_secs(&mtime),
                _ => false,
            };

            if mtime_changed {
                header();
                println!("  M {} Modification time changed", path.to_string_lossy().red());
                continue;
            }
        }

        match entry {
            Entry::Dir { .. } => {
                if !metadata.is_dir() {
                    header();
                    println!("  T {} Not a directory", path.to_string_lossy().red());
                    continue;
                }
            },

            Entry::File { ref md5, .. } => {
                if !metadata.is_file() {
                    header();
                    println!("  T {} Not a regular file", path.to_string_lossy().red());
                    continue;
                }

                if !opts.no_integrity {
                    match file_md5(path) {
                        Ok(real_md5) => {
                            if &real_md5 != md5 {
                                header();
                                println!("  C {} Content changed", path.to_string_lossy().red());
                                continue;
                            }
                        },
                        Err(err) => {
                            println!("  X {} {}", path.to_string_lossy().red(), err.to_string());
                        },
                    }
                }
            },

            Entry::Sym { ref target, .. } => {
                if !metadata.file_type().is_symlink() {
                    header();
                    println!("  T {} Not a symbolic link", path.to_string_lossy().red());
                    continue;
                }

                match path.read_link() {
                    Ok(real_target) => {
                        if *target != real_target {
                            header();
                            println!("  C {} Symlink changed", path.to_string_lossy().red());
                            continue;
                        }
                    },
                    Err(err) => {
                        header();
                        println!("  X {} {}", path.to_string_lossy().red(), err.to_string());
                        continue;
                    },
                }

                if let Err(err) = path.canonicalize() {
                    header();
                    if err.kind() == std::io::ErrorKind::NotFound {
                        println!("  X {} Dangling symbolic link", path.to_string_lossy().red());
                    } else {
                        println!("  X {} {}", path.to_string_lossy().red(), err.to_string());
                    }
                    continue;
                }
            },
        }
    }
}

fn epoch_secs(moment: &SystemTime) -> u64 {
    moment.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn file_md5(path: &Path) -> io::Result<String> {
    let mut reader = io::BufReader::new(fs::File::open(path)?);
    let mut hasher = Hasher::new(Algorithm::MD5);
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(hex::encode(hasher.finish()))
        }
        hasher.write_all(chunk)?;

        let n = chunk.len();
        reader.consume(n);
    }
}
