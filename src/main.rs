mod opts;

use std::{io::Read, process, thread, sync::{Arc, Mutex}};

use image::{EncodableLayout, ImageBuffer, Rgba};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};
use structopt::StructOpt;
use tracing;
use tracing_subscriber::EnvFilter;
use shellexpand;

use crate::opts::Opts;

slint::slint! {
    import { Button, LineEdit } from "std-widgets.slint";

    MainWindow := Window {
        property <bool> loading;
        property <image> display_image;
        property <string> file_name;

        callback save_to_file(string);

        title: (loading ? "Loading..." : (file_name + " (" + display_image.width + "px x " + display_image.height + "px)")) + " - kiraz";
        preferred_width: 800px;
        preferred_height: 800px;

        VerticalLayout {
            padding: 8px;

            Image {
                source: display_image;
                preferred_height: 768px;
                image_fit: contain;
            }
            HorizontalLayout {
                height: 32px;

                FilePath := LineEdit {
                    placeholder_text: "Path to save the file at";
                }
                Button {
                    text: "Save to file";
                    clicked => { save_to_file(FilePath.text); }
                }
            }
        }
    }
}

type ImageData = Arc<Mutex<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>>>;

/// Sets up the logging. Only call this once, at the start.
fn setup_logging() -> () {
    let env_filter =
        EnvFilter::try_from_env("KIRAZ_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("warn"));
    let subscriber = tracing_subscriber::fmt::fmt()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");
}

fn load_image(opts: Opts, window_weak: slint::Weak<MainWindow>, image_save: ImageData) -> anyhow::Result<()> {
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

    let _ = image_save.lock().unwrap().insert(image);

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

    let image: ImageData = Arc::new(Mutex::new(None));
    let image_clone = image.clone();
    thread::spawn(move || {
        load_image(opts, window_weak, image_clone).unwrap_or_else(|error| {
            tracing::error!("Failed to load the image: {:?}", error);
            process::exit(1);
        });
    });

    window.on_save_to_file(move |path| {
        let image = image.clone();
        // Save the image to a file in a background thread
        thread::spawn(move || {
            let image = image.lock().unwrap();
            tracing::debug!("Image has been set: {}", image.is_some());
            image.as_ref().map(|image| {
                tracing::debug!("Saving image...");
                let target_path = shellexpand::full(path.as_str());
                match target_path {
                    Ok(target_path) => {
                        tracing::debug!("Saving to path {}", &target_path);
                        image.save(target_path.as_ref()).unwrap();
                    },
                    Err(err) => {
                        tracing::error!("Failed to calculate the path to save to: {:?}", err);
                    },
                }
                image.save(path.as_str()).unwrap();
                tracing::debug!("Done!");
            });
        });
    });

    Ok(window.run())
}
