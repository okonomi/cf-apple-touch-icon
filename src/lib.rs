use worker::*;
use image::ImageOutputFormat;
use std::io::Cursor;

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

    let bytes = std::include_bytes!("../res/icon.jpg");
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg).unwrap();

    let img2 = img.resize(200, 200, image::imageops::FilterType::Triangle);

    let mut result_buf: Vec<u8> = Vec::new();
    img2.write_to(&mut Cursor::new(&mut result_buf), ImageOutputFormat::Png).expect("io error");

    let response = Response::from_bytes(result_buf).unwrap();
    let mut headers = Headers::new();
    headers.set("content-type", "image/png")?;
    Ok(response.with_headers(headers))
}
