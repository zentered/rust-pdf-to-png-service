use google_cloud_default::WithAuthExt;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::Media;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use google_cloud_storage::http::objects::upload::UploadType;
use google_cloud_storage::http::Error;
use std::env;
use tracing::debug;

#[tracing::instrument]
pub async fn download(filename: &str) -> Result<Vec<u8>, Error> {
    debug!("downloading file {:?}", filename);
    let config = ClientConfig::default().with_auth().await.unwrap();
    let client = Client::new(config);

    let pdf_buffer = client
        .download_object(
            &GetObjectRequest {
                bucket: env::var("SOURCE_BUCKET").unwrap().to_string(),
                object: filename.to_string(),
                ..Default::default()
            },
            &Range::default(),
            None,
        )
        .await
        .unwrap();

    Ok(pdf_buffer)
}

// #[tracing::instrument]
pub async fn upload(filename: &str, file: Vec<u8>) -> Result<(), Error> {
    debug!("uploading file {:?}", filename);

    let config = ClientConfig::default().with_auth().await.unwrap();
    let client = Client::new(config);

    // Upload the file
    let mut media = Media::new(filename.to_string());
    media.content_type = match filename.rsplit_once('.').unwrap().1 {
        "webp" => "image/webp".into(),
        "png" => "image/png".into(),
        "jpg" => "image/jpeg".into(),
        _ => "application/octet-stream".into(),
    };

    let uploaded = client
        .upload_object(
            &UploadObjectRequest {
                bucket: env::var("DEST_BUCKET").unwrap().to_string(),
                ..Default::default()
            },
            file,
            &UploadType::Simple(media),
            None,
        )
        .await;

    println!("uploaded: {:?}", uploaded);
    Ok(())
}
