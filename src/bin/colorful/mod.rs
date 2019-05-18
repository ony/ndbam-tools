use clap::arg_enum;
pub use colored::*;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ColorWhen {
        Auto = 0,
        Always,
        Never,
    }
}

impl ColorWhen {
    pub fn force(&self) {
        match self {
            ColorWhen::Auto => {
                if !atty::is(atty::Stream::Stdout) {
                    colored::control::set_override(false);
                }
            }
            ColorWhen::Always => {
                colored::control::set_override(true);
            }
            ColorWhen::Never => {
                colored::control::set_override(false);
            }
        }
    }
}
