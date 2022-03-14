///! The CLI opts for kiraz.
use structopt::StructOpt;

#[derive(StructOpt)]
/// Edit and upload your screenshots, and other images.
/// 
/// Examples
/// 
/// To open a file:
///   kiraz my-image.png
/// Screenshot the whole desktop with grim:
///   grim | kiraz -
/// Screenshot a single desktop with grim and slurp:
///   grim -g (slurp) | kiraz -
pub struct Opts {
  /// The file to open. Use - to read from stdin when piping.
  pub file: std::path::PathBuf,
}
