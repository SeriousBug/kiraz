use std::io::Read;

use image::EncodableLayout;
use slint::{SharedPixelBuffer, Rgba8Pixel, Image};
use tracing;
use tracing_subscriber::{Registry, EnvFilter, layer::SubscriberExt};


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
/// 
/// Thanks to Luca Palmieri: https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/
fn setup_logging() -> () {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // The `with` method is provided by `SubscriberExt`, an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default().with(env_filter);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");

}

fn main() {
    setup_logging();

    let mut image_data: Vec<u8> = vec![];
    std::io::stdin().read_to_end(&mut image_data).expect("Unable to read data from stdin");
    println!("Read {} bytes of data", image_data.len());
    tracing::debug!("Read {} bytes of data", image_data.len());
    let mut image_data_cursor = std::io::Cursor::new(&image_data);

    let img = image::io::Reader::new(&mut image_data_cursor).with_guessed_format().unwrap();
    tracing::debug!("Read {:?} image from stdin", &img.format());
    println!("Read {:?} image from stdin", &img.format());
    
    let im_ = img.decode().unwrap().to_rgba8();

    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(im_.as_bytes(), im_.width(), im_.height());
    println!("Image size is {} by {} pixels", im_.width(), im_.height());

    let window = MainWindow::new();
    let ui_image = Image::from_rgba8(buffer);
    window.set_display_image(ui_image);
    window.run();
}
