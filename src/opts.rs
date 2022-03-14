///! The CLI opts for swappers.
use structopt::StructOpt;

#[derive(StructOpt)]
/// The top level of all CLI opts used.
pub struct Opts {
  /// The file to open. Use - to read from stdin when piping.
  pub file: std::path::PathBuf,
}
