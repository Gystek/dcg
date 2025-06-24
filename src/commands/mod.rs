use clap::Subcommand;

pub(crate) mod init;

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// initialize a new dcg repository
    Init {
        /// the name to use for the initial branch
        /// instead of 'master' (by default) or the
        /// branch name as defined in the user
        /// configuration.
        #[arg(short = 'b', long)]
        initial_branch: Option<String>,

        directory: Option<String>,
    },
}
