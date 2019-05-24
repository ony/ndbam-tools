extern crate structopt;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use ndbam::*;
use ndbam::contents::*;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use bytesize::ByteSize;

mod colorful;
mod env_opts;
use colorful::*;
use env_opts::*;

const DEFAULT_REPO_PATH : &'static str = "/var/db/paludis/repositories/installed";

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

    /// Exclude file (can be specified multiple times)
    #[structopt(long = "exclude", raw(conflicts_with_all = r#"&["no_contents", "files"]"#))]
    excludes: Vec<PathBuf>,

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

trait ContentFilter {
    fn skip_entry(&self, _: &Entry) -> bool {
        false
    }

    fn should_report_matches(&self) -> bool {
        false
    }
}

enum FileFilter<'s> {
    Everything,
    Include(HashSet<&'s Path>),
    Exclude(HashSet<&'s Path>),
}

impl<'s> ContentFilter for FileFilter<'s> {
    fn skip_entry(&self, entry: &Entry) -> bool {
        match self {
            FileFilter::Everything => false,
            FileFilter::Include(whitelist) => !whitelist.contains(entry.path()),
            FileFilter::Exclude(blacklist) => blacklist.contains(entry.path()),
        }
    }
    fn should_report_matches(&self) -> bool {
        match self {
            FileFilter::Everything => false,
            FileFilter::Include(_) => true,
            FileFilter::Exclude(_) => false,
        }
    }
}

fn main() {
    let opts =  Opts::from_args();
    opts.color.force();


    let content_filter = if !opts.files.is_empty() {
        let mut files : HashSet<&Path> = HashSet::new();
        for ref file in &opts.files {
            files.insert(file);
        }
        FileFilter::Include(files)
    } else if !opts.excludes.is_empty() {
        let mut files : HashSet<&Path> = HashSet::new();
        for ref file in &opts.excludes {
            files.insert(file);
        }
        FileFilter::Exclude(files)
    } else {
        FileFilter::Everything
    };

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
            let size = check_contents(&opts, &content_filter, &pkg, &mut reporter);
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

fn check_contents(opts: &Opts, filter: &ContentFilter, pkg: &PackageView, reporter: &mut impl ContentReporter) -> u64 {
    let root = &opts.env.root;
    let mut size = 0;
    for ref entry in pkg.contents() {
        let path = entry.path();
        let real_path = root.real_path(entry.path()).unwrap();
        if filter.skip_entry(&entry) {
            continue;
        }

        if opts.verbose {
            reporter.dump_entry(entry);
        } else if filter.should_report_matches() {
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
                    match file_hash(Algorithm::MD5, real_path) {
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

                if let Err(err) = root.canonicalize_to_real(path) {
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

fn epoch_secs(moment: &SystemTime) -> u64 {
    moment.duration_since(UNIX_EPOCH).unwrap().as_secs()
}
