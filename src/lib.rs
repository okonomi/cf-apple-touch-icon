use image::{DynamicImage, ImageOutputFormat};
use regex::Regex;
use std::io::Cursor;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let re = Regex::new(r"^/apple-touch-icon(-(\d+)x(\d+))?(-precomposed)?\.png").unwrap();
    let path = req.path();
    let caps = re.captures(&path).unwrap();

    let width: u32 = caps.get(2).map_or("60", |m| m.as_str()).parse().unwrap();
    let height: u32 = caps.get(3).map_or("60", |m| m.as_str()).parse().unwrap();
    let icon = generate_icon(width, height);

    let mut result_buf: Vec<u8> = Vec::new();
    icon.write_to(&mut Cursor::new(&mut result_buf), ImageOutputFormat::Png)
        .expect("io error");

    let response = Response::from_bytes(result_buf).unwrap();
    let mut headers = Headers::new();
    headers.set("content-type", "image/png")?;
    Ok(response.with_headers(headers))
}

fn generate_icon(width: u32, height: u32) -> DynamicImage {
    let bytes = std::include_bytes!("../res/icon.jpg");
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg).unwrap();

    let img2 = img.resize(width, height, image::imageops::FilterType::Triangle);
    img2
}
