pub mod second;

use clap::ValueEnum;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, ValueEnum)]
pub enum Part {
    One,
    Two,
}
