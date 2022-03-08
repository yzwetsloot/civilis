use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
pub struct Args {
    /// Set request connect timeout value
    #[clap(long, short, default_value = "10")]
    pub timeout: u64,

    /// Turn debugging information on
    #[clap(long, short)]
    pub debug: bool,

    /// Set base URL to start with
    #[clap(long, short, default_value = "https://github.com")]
    pub url: String,

    /// Set recursive visitation depth
    #[clap(long)]
    pub depth: u16,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
