mod opts;

use std::io::Read;

use image::EncodableLayout;
use slint::{SharedPixelBuffer, Rgba8Pixel, Image};
use structopt::StructOpt;
use tracing;
use tracing_subscriber::EnvFilter;

use crate::opts::Opts;


slint::slint! {
    MainWindow := Window {
        property <image> display_image;
        property <string> file_name;

         title: file_name + " (" + display_image.width + "px x " + display_image.height + "px) - swappers";

        VerticalLayout {
            width: 1024px;

            Image {
                source: display_image;
                width: parent.width;
                height: 768px;
                image_fit: contain;
            }
        }
    }
}

/// Sets up the logging. Only call this once, at the start.
fn setup_logging() -> () {
    let env_filter = EnvFilter::try_from_env("SWAPPERS_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("warn"));
    let subscriber = tracing_subscriber::fmt::fmt().with_env_filter(env_filter).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");
}



fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    setup_logging();

    // Select the source for the input image
    let mut data_source: Box<dyn Read> = if let Some("-") = opts.file.to_str() {
        tracing::debug!("Reading data from stdin");
        Box::new(std::io::stdin())
    } else {
        tracing::debug!("Reading data from file {:?}", opts.file);
        Box::new(std::fs::File::open(opts.file)?)
    };

    // Read the image data
    let mut image_data: Vec<u8> = vec![];
    data_source.read_to_end(&mut image_data)?;
    tracing::debug!("Read {} bytes of data", image_data.len());

    // Decode the image data, and convert it to a raw format that we can load into the UI
    let mut image_data_cursor = std::io::Cursor::new(&image_data);
    let image_reader = image::io::Reader::new(&mut image_data_cursor).with_guessed_format()?;
    tracing::debug!("Found {:?} format", image_reader.format());
    let image = image_reader.decode()?.to_rgba8();
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(image.as_bytes(), image.width(), image.height());

    // Create the window, set the image, and start the UI
    let window = MainWindow::new();
    let ui_image = Image::from_rgba8(buffer);
    window.set_display_image(ui_image);
    Ok(window.run())
}
