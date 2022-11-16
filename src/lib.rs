use image::{DynamicImage, ImageOutputFormat};
use regex::Regex;
use std::io::Cursor;
use worker::*;

mod utils;

struct Icon {
    width: u32,
    height: u32,
    precomposed: bool,
}

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

    let icon = match parse_icon_path(&req.path()) {
        Ok(icon) => icon,
        Err(e) => return Response::error(format!("{:?}", e), 400),
    };

    if !validate_icon(&icon) {
        return Response::error("err", 400);
    }

    let icon_img = generate_icon(&icon);

    let response = make_response(&icon_img);

    Ok(response)
}

fn parse_icon_path(path: &str) -> Result<Icon> {
    let re = Regex::new(r"^/apple-touch-icon(-(\d+)x(\d+))?(-precomposed)?\.png").unwrap();
    let caps = re.captures(&path).ok_or("erorr".to_owned())?;

    let width: u32 = caps.get(2).map_or("60", |m| m.as_str()).parse().unwrap();
    let height: u32 = caps.get(3).map_or("60", |m| m.as_str()).parse().unwrap();
    let precomposed: bool = caps.get(4).map_or("", |m| m.as_str()) == "-precomposed";
    Ok(Icon {
        width,
        height,
        precomposed,
    })
}

fn validate_icon(icon: &Icon) -> bool {
    if icon.width < 1 || icon.width > 500 {
        return false;
    }
    if icon.height < 1 || icon.height > 500 {
        return false;
    }
    if icon.width != icon.height {
        return false;
    }

    true
}

fn generate_icon(icon: &Icon) -> DynamicImage {
    let bytes = std::include_bytes!("../res/icon.jpg");
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg).unwrap();

    let img2 = img.resize(
        icon.width,
        icon.height,
        image::imageops::FilterType::Triangle,
    );
    img2
}

fn make_response(icon_img: &DynamicImage) -> Response {
  let mut result_buf: Vec<u8> = Vec::new();
  icon_img
      .write_to(&mut Cursor::new(&mut result_buf), ImageOutputFormat::Png)
      .expect("io error");

  let response = Response::from_bytes(result_buf).unwrap();
  let mut headers = Headers::new();
  headers.set("content-type", "image/png");
  response.with_headers(headers)
}
