mod opts;

use std::{io::Read, process, thread};

use image::EncodableLayout;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};
use structopt::StructOpt;
use tracing;
use tracing_subscriber::EnvFilter;

use crate::opts::Opts;

slint::slint! {
    MainWindow := Window {
        property <bool> loading;
        property <image> display_image;
        property <string> file_name;

        title: (loading ? "Loading..." : (file_name + " (" + display_image.width + "px x " + display_image.height + "px)")) + " - kiraz";

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
    let env_filter =
        EnvFilter::try_from_env("KIRAZ_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("warn"));
    let subscriber = tracing_subscriber::fmt::fmt()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");
}

fn load_image(opts: Opts, window_weak: slint::Weak<MainWindow>) -> anyhow::Result<()> {
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
    tracing::debug!("Found format {:?}", image_reader.format());
    let image = image_reader.decode()?.to_rgba8();
    tracing::debug!("Image is {} by {} pixels", image.width(), image.height());
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        image.as_bytes(),
        image.width(),
        image.height(),
    );

    tracing::debug!("Image ready");
    slint::invoke_from_event_loop(move || {
        let image = Image::from_rgba8(buffer);
        let window = window_weak.unwrap();
        window.set_display_image(image);
        window.set_loading(false);
    });
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    setup_logging();

    let window = MainWindow::new();
    window.set_file_name(SharedString::from(opts.file.to_string_lossy().to_string()));
    window.set_loading(true);
    let window_weak = window.as_weak();

    thread::spawn(move || {
        load_image(opts, window_weak).unwrap_or_else(|error| {
            tracing::error!("Failed to load the image: {:?}", error);
            process::exit(1);
        });
    });

    Ok(window.run())
}
