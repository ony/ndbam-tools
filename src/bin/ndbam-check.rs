extern crate structopt;

#[macro_use]
extern crate clap;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use ndbam::*;
use ndbam::contents::*;
use crypto_hash::{Algorithm, Hasher};
use structopt::StructOpt;
use structopt::clap::AppSettings;
use colored::*;
use bytesize::ByteSize;


arg_enum! {
    #[derive(PartialEq, Debug)]
    enum ColorWhen {
        Auto = 0,
        Always,
        Never,
    }
}

impl ColorWhen {
    fn force(&self) {
        match self {
            ColorWhen::Auto => {
                if !atty::is(atty::Stream::Stdout) {
                    colored::control::set_override(false);
                }
            },
            ColorWhen::Always => {
                colored::control::set_override(true);
            },
            ColorWhen::Never => {
                colored::control::set_override(false);
            },
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
struct Opts {
    /// Location of database
    #[structopt(short, long, name = "PATH", default_value = "/var/db/paludis/repositories/installed")]
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

    /// Show sizes of all packages (inhibited by --no-contents)
    #[structopt(short = "s", long = "show-size", raw(conflicts_with = r#""no_contents""#))]
    show_size: bool,

    /// Colorize output?
    #[structopt(long, name = "WHEN", default_value = "auto", raw(possible_values = "&ColorWhen::variants()", case_insensitive = "true"))]
    color: ColorWhen,

    #[structopt(short, long)]
    verbose: bool,
}

fn main() {
    let opts = Opts::from_args();
    opts.color.force();

    let reg = NDBAM::new(&opts.location);
    let mut total_size = 0u64;
    reg.all_packages()
        .map(|iter|
             for pkg in iter {
                 let mut reporter = ConsolePackageReporter::new(&pkg);
                 if opts.verbose {
                     reporter.header()
                 }
                 if !opts.no_contents {
                     let size = check_contents(&opts, &pkg, &mut reporter);
                     total_size += size;
                     if opts.show_size { reporter.header() }  // force report
                     if reporter.any_reports() {
                         println!("  # {}: {}", "Size".bold(), ByteSize::b(size));
                     }
                 }
             }
        );
    if opts.show_size && total_size > 0 {
        println!("");
        println!("  # {}: {}", "Total size".bold(), ByteSize::b(total_size));
    }
}

trait ContentReporter {
    fn note(&mut self, content_entry: &Entry, class: char, note: &str);
    fn err<E: ToString>(&mut self, content_entry: &Entry, err: E) {
        self.note(content_entry, 'X', &err.to_string())
    }
}

struct ConsolePackageReporter<'p> {
    pkg: &'p PackageView,
    any_reports: bool,
}

impl<'p> ConsolePackageReporter<'p> {
    fn new(pkg: &PackageView) -> ConsolePackageReporter {
        ConsolePackageReporter { pkg, any_reports: false }
    }

    fn any_reports(&self) -> bool { self.any_reports }

    fn header(&mut self) {
        if self.any_reports { return }
        self.any_reports = true;
        println!("{}:{}", self.pkg.full_name(), self.pkg.slot().unwrap_or("0"));
        if let Ok(summary) = self.pkg.read_key("SUMMARY") {
            if !summary.trim_end().is_empty() {
                println!("  # {}: {}", "Summary".bold(), summary.trim_end());
            }
        }
    }
}

impl<'p> ContentReporter for ConsolePackageReporter<'p> {
    fn note(&mut self, content_entry: &Entry, class: char, note: &str) {
        self.header();
        println!("  {} {} {}", class, content_entry.path().to_string_lossy().red(), note);
    }
}

fn check_contents(opts: &Opts, pkg: &PackageView, reporter: &mut impl ContentReporter) -> u64 {
    let mut size = 0;
    for ref entry in pkg.contents() {
        if opts.verbose {
            println!("  {:?}", entry);
        }

        let path = entry.path();
        let metadata = match path.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    reporter.note(entry, 'X', "Does not exist");
                } else {
                    reporter.err(entry, err);
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
                reporter.note(entry, 'X', "Modification time changed");
                continue;
            }
        }

        match entry {
            Entry::Dir { .. } => {
                if !metadata.is_dir() {
                    reporter.note(entry, 'T', "Not a directory");
                    continue;
                }
            },

            Entry::File { ref md5, .. } => {
                if !metadata.is_file() {
                    reporter.note(entry, 'T', "Not a regular file");
                    continue;
                }

                if !opts.no_integrity {
                    match file_md5(path) {
                        Ok(real_md5) => {
                            if &real_md5 != md5 {
                                reporter.note(entry, 'C', "Content changed");
                                continue;
                            }
                        },
                        Err(err) => {
                            reporter.err(entry, err);
                        },
                    }
                }
            },

            Entry::Sym { ref target, .. } => {
                if !metadata.file_type().is_symlink() {
                    reporter.note(entry, 'T', "Not a symbolic link");
                    continue;
                }

                match path.read_link() {
                    Ok(real_target) => {
                        if *target != real_target {
                            reporter.note(entry, 'C', "Symlink changed");
                            continue;
                        }
                    },
                    Err(err) => {
                        reporter.err(entry, err);
                        continue;
                    },
                }

                if let Err(err) = path.canonicalize() {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        reporter.note(entry, 'X', "Dangling symbolic link");
                    } else {
                        reporter.err(entry, err);
                    }
                }
            },
        }

        // Count only content confirmed to be owned by package
        size += metadata.len();
    }
    size
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
