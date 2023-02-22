use image::DynamicImage;
use pdfium_render::prelude::*;
use tracing::info;

pub fn convert(pdf_buffer: Vec<u8>) -> DynamicImage {
    info!("convert started...");

    let render_config = PdfRenderConfig::new()
        .set_target_width(3000)
        .set_maximum_height(3000)
        .rotate_if_landscape(PdfBitmapRotation::Degrees90, true);

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib/")).unwrap(),
    );
    let document = pdfium.load_pdf_from_byte_vec(pdf_buffer, None).unwrap();
    let mut page = document.pages().first().unwrap();
    page.flatten().expect("flatten failed");
    let original_png = page.render_with_config(&render_config).unwrap().as_image();

    return original_png;
}
