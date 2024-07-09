use clap::Parser;

#[derive(Parser, Default, Debug)]
#[clap(author = "Shane Poppleton", version, about = "Forge site move command")]
pub struct Args {
    /// Specifies the source directory to copy
    #[clap(short, long)]
    pub dir: String,

    /// Specifies the target server
    #[clap(short, long)]
    pub server: String,

    /// Specifies the target folder
    #[clap(short, long)]
    pub target: String,
}
