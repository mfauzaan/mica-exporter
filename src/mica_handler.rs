use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use image::ImageOutputFormat;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use mica;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{fs::File, io::Cursor, io::Read, io::Write};
use zip::{write::FileOptions, write::ZipWriter};

use crate::storage::{s3::S3, Storage};

pub const BUCKET_REGION: &str = "ap-southeast-1";

pub async fn extract_layers_and_save(event: Request) -> Result<Response<Body>, Error> {
    let region = RegionProviderChain::default_provider().or_else(BUCKET_REGION);
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    let (bucket, bucket_key) = get_bucket_info(&event)?;

    let client = aws_sdk_s3::Client::new(&config);

    let storage = S3::new(client, bucket.to_string());
    let source = storage.get_object(bucket_key).await?;

    let dev = mica::compositor::dev::GpuHandle::new()
        .await
        .expect("Unable to create GpuHandle");

    let tmp_dir = tempfile::Builder::new()
        .prefix("mica")
        .tempdir()
        .expect("Failed to create temporary directory");

    let tmp_path = tmp_dir.path().join("mica.zip");
    let zip_writer = File::create(&tmp_path).expect("Unable to create file");

    let mut zip = ZipWriter::new(zip_writer);

    let app = mica::app::App::new(dev);

    let (file, gpu_textures, target) = app
        .load_file_from_bytes(source)
        .await
        .expect("Unable to load file");

    let image_buffers = app
        .extract_image_buffers(&file, &gpu_textures, target)
        .await;

    let image_buffers: Vec<Vec<u8>> = image_buffers
        .into_par_iter()
        .map(|image_buffer| {
            println!("Writing image to buffer");
            let mut buf = Cursor::new(Vec::new());

            image_buffer
                .write_to(&mut buf, ImageOutputFormat::Png)
                .unwrap();

            let inner_vec = buf.into_inner();
            println!("Finished writing image to buffer");

            inner_vec
        })
        .collect();

    for (index, image_buffer) in image_buffers.iter().enumerate() {
        println!("Writing image_{}.png", index);
        let file_path = format!("image_{}.png", index);

        zip.start_file(file_path, FileOptions::default()).unwrap();

        zip.write_all(&image_buffer[..])
            .expect("Unable to write to zip");
        println!("Finished writing image_{}.png", index);
    }

    zip.finish().expect("Unable to finish zip");

    let mut buffer = Vec::new();
    let mut zip_file = File::open(tmp_path)?;
    zip_file.read_to_end(&mut buffer)?;

    storage.save_object(&format!("{}/mica.zip", bucket_key), &buffer).await?;

    let resp = Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Body::from("Request processed successfully!"))
        .map_err(Box::new)?;

    Ok(resp)
}

fn get_bucket_info(event: &Request) -> Result<(&str, &str), &'static str> {
    let bucket_key = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("bucket_key"))
        .expect("Missing bucket_path parameter");

    let bucket = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("bucket"))
        .expect("Missing bucket_path parameter");

    Ok((bucket, bucket_key))
}
