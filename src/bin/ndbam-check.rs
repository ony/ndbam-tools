extern crate structopt;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use ndbam::*;
use ndbam::contents::*;
use crypto_hash::{Algorithm, Hasher};
use structopt::clap::AppSettings;
use structopt::StructOpt;
use bytesize::ByteSize;

mod colorful;
mod env_opts;
use colorful::*;
use env_opts::*;

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
struct Opts {
    #[structopt(flatten)]
    env: EnvOpts,

    /// Skip checking package contents
    #[structopt(long = "no-contents")]
    no_contents: bool,

    /// Allow modification time changes
    #[structopt(long = "allow-mtime")]
    allow_mtime: bool,

    /// Skip file integrity checking
    #[structopt(long = "no-integrity")]
    no_integrity: bool,

    /// Check file (can be specified multiple times)
    #[structopt(long = "file", raw(conflicts_with = r#""no_contents""#))]
    files: Vec<PathBuf>,

    /// Show sizes of all packages (inhibited by --no-contents)
    #[structopt(short = "s", long = "show-size", raw(conflicts_with = r#""no_contents""#))]
    show_size: bool,

    /// Colorize output?
    #[structopt(long, name = "WHEN", default_value = "auto", raw(possible_values = "&ColorWhen::variants()", case_insensitive = "true"))]
    color: ColorWhen,

    #[structopt(short, long)]
    verbose: bool,

    /// Package names to inspect (by default whole database)
    #[structopt(name = "PACKAGE NAMES")]
    names: Vec<String>,
}

fn main() {
    let opts = {
        let mut raw_opts = Opts::from_args();
        raw_opts.env.canonicalize();
        raw_opts
    };
    opts.color.force();

    let mut files : HashSet<&Path> = HashSet::new();
    for file in &opts.files {
        files.insert(file.as_path());
    }

    let reg = opts.env.ndbam();
    let mut total_size = 0u64;
    let mut missing_packages = false;
    let mut any_problems = false;
    let mut handle_package = |pkg| {
        let mut reporter = ConsolePackageReporter::new(&pkg);
        if opts.verbose {
            reporter.header()
        }
        if !opts.no_contents {
            let size = check_contents(&opts, &files, &pkg, &mut reporter);
            any_problems = any_problems || reporter.any_problems;
            total_size += size;
            if opts.show_size { reporter.header() }  // force report
            if reporter.any_reports() {
                println!("  # {}: {}", "Size".bold(), ByteSize::b(size));
            }
        }
    };

    if opts.names.is_empty() {
        reg.all_packages().map(|iter| for pkg in iter { handle_package(pkg) });
    } else {
        for ref name in &opts.names {
            if let Some(iter) = reg.versions_of(name) {
                for pkg in iter { handle_package(pkg) }
            } else {
                println!("{} - {}", name, "Not found".red().bold());
                missing_packages = true;
            }
        }
    }

    if opts.show_size && total_size > 0 {
        println!("");
        println!("  # {}: {}", "Total size".bold(), ByteSize::b(total_size));
    }

    if any_problems {
        std::process::exit(1);
    } else if missing_packages {
        std::process::exit(2);
    }
}

trait ContentReporter {
    fn note(&mut self, content_entry: &Entry, class: char, note: &str);
    fn err<E: ToString>(&mut self, content_entry: &Entry, err: E) {
        self.note(content_entry, 'X', &err.to_string())
    }
    fn dump_entry(&mut self, content_entry: &Entry);
}

struct ConsolePackageReporter<'p> {
    pkg: &'p PackageView,
    any_reports: bool,
    any_problems: bool,
}

impl<'p> ConsolePackageReporter<'p> {
    fn new(pkg: &PackageView) -> ConsolePackageReporter {
        ConsolePackageReporter { pkg, any_reports: false, any_problems: false }
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

        // TODO: use enum for class or separate method
        if class != '#' {
            self.any_problems = true;
        }
    }
    fn dump_entry(&mut self, content_entry: &Entry) {
        self.header();
        println!("  # {:?}", content_entry);
    }
}

fn check_contents(opts: &Opts, files: &HashSet<&Path>, pkg: &PackageView, reporter: &mut impl ContentReporter) -> u64 {
    let mut size = 0;
    for ref entry in pkg.contents() {
        let path = entry.path();
        let real_path = entry.path_in(&opts.env.root);
        if !files.is_empty() && !files.contains(path) {
            continue;
        }

        if opts.verbose {
            reporter.dump_entry(entry);
        } else if !files.is_empty() {
            reporter.note(entry, '#', "Match");
        }

        let metadata = match real_path.symlink_metadata() {
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
                reporter.note(entry, 'M', "Modification time changed");
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
                    match file_md5(&real_path) {
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

                // Count only file content confirmed to be owned by package
                size += metadata.len();
            },

            Entry::Sym { ref target, .. } => {
                if !metadata.file_type().is_symlink() {
                    reporter.note(entry, 'T', "Not a symbolic link");
                    continue;
                }

                match real_path.read_link() {
                    Ok(actual_target) => {
                        if *target != actual_target {
                            reporter.note(entry, 'C', "Symlink changed");
                            continue;
                        }
                    },
                    Err(err) => {
                        reporter.err(entry, err);
                        continue;
                    },
                }

                if let Err(err) = root_canonicalize(&opts.env.root, path) {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        reporter.note(entry, 'X', "Dangling symbolic link");
                        continue;
                    } else {
                        reporter.err(entry, err);
                    }
                }
            },
        }
    }
    size
}

fn root_canonicalize(root: &Path, target: &Path) -> io::Result<PathBuf> {
    debug_assert!(target.is_absolute());
    let mut result = root.to_path_buf();
    let mut level = 0;
    let mut components = target.components();
    components.next();  // skip leading root indicator
    for component in components {
        result.push(component);
        level += 1;
        if let Ok(target) = result.read_link() {
            if target.is_absolute() {
                // "Reset" to root
                while level > 0 {
                    result.pop();
                    level -= 1;
                }
                result.push(target);
            }
            // I'm lazy to resolving relative alongside with proper "reset" to root
        }
    }

    result.canonicalize()
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
