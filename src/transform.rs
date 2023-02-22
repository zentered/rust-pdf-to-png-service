use crate::Format;
use libvips::ops;
use libvips::Result;
use libvips::VipsApp;
use libvips::VipsImage;
use tracing::info;

pub fn init() -> VipsApp {
    info!("init started...");
    let app: VipsApp = VipsApp::new("libvips", false).expect("Cannot initialize libvips");
    app.concurrency_set(2);
    return app;
}

pub fn load_image(original_png: &Vec<u8>) -> VipsImage {
    info!("vips::load_image started...");

    let image = VipsImage::new_from_buffer(original_png, "").expect("Cannot load image");
    return image;
}

pub fn transform(image: &VipsImage, format: Format) -> Result<Vec<u8>> {
    info!("vips::ransform started...");
    match format {
        Format::Original => {
            let processed = ops::copy(image).expect("Vips copy failed");
            let options = ops::PngsaveBufferOptions {
                q: 90,
                compression: 8,
                strip: true,
                bitdepth: 8,
                ..ops::PngsaveBufferOptions::default()
            };
            ops::pngsave_buffer_with_opts(&processed, &options)
        }
        Format::Preview => {
            let image_width = image.get_width();
            let target_width = 1920;
            let scale = target_width as f64 / image_width as f64;
            let resized = ops::resize(image, scale).expect("Vips resize failed");
            let options = ops::PngsaveBufferOptions {
                q: 75,
                strip: true,
                compression: 8,
                interlace: true,
                bitdepth: 8,
                ..ops::PngsaveBufferOptions::default()
            };
            ops::pngsave_buffer_with_opts(&resized, &options)
        }
        Format::Thumbnail => {
            let resized = ops::thumbnail_image(image, 1024).expect("Vips thumbnail failed");
            let options = ops::PngsaveBufferOptions {
                q: 70,
                strip: true,
                interlace: true,
                compression: 8,
                bitdepth: 8,
                ..ops::PngsaveBufferOptions::default()
            };
            ops::pngsave_buffer_with_opts(&resized, &options)
        }
        Format::WebPLossless => {
            let processed = ops::copy(image).expect("Vips copy failed");
            let options = ops::WebpsaveBufferOptions {
                q: 100,
                lossless: true,
                strip: true,
                ..ops::WebpsaveBufferOptions::default()
            };
            ops::webpsave_buffer_with_opts(&processed, &options)
        }
        Format::WebP => {
            let image_width = image.get_width();
            let target_width = 1920;
            let scale = target_width as f64 / image_width as f64;
            let processed = ops::resize(image, scale).expect("Vips resize failed");
            let options = ops::WebpsaveBufferOptions {
                q: 75,
                strip: true,
                ..ops::WebpsaveBufferOptions::default()
            };
            ops::webpsave_buffer_with_opts(&processed, &options)
        }
    }
}
