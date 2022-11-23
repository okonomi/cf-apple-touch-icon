use image::{DynamicImage, ImageOutputFormat};
use regex::Regex;
use std::io::Cursor;
use worker::*;

mod utils;

struct Icon {
    width: u32,
    height: u32,
}

impl Icon {
    fn validate(&self) -> Result<()> {
        if self.width < 1 || self.width > 500 {
            return Err(Error::from("invalid width"));
        }
        if self.height < 1 || self.height > 500 {
            return Err(Error::from("invalid height"));
        }
        if self.width != self.height {
            return Err(Error::from("invalid size"));
        }

        Ok(())
    }
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

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let icon = match parse_icon_path(&req.path().trim_start_matches("/")) {
        Ok(icon) => icon,
        Err(e) => return Response::error(e.to_string(), 400),
    };

    if let Err(e) = icon.validate() {
        return Response::error(e.to_string(), 403);
    }

    let cache = Cache::default();
    let key = req.url()?.to_string();
    console_debug!("key = {}", key);
    let response;
    if let Some(resp) = cache.get(&key, true).await? {
        console_debug!("Cache HIT!");
        response = resp;
    } else {
        console_debug!("Cache MISS!");
        let source_icon = load_source_icon(&env).await?;
        let icon_img = generate_icon(&icon, &source_icon);
        response = make_response(&icon_img)?;
    }

    Ok(response)
}

fn parse_icon_path(path: &str) -> Result<Icon> {
    let re = Regex::new(r"^apple-touch-icon(-(\d+)x(\d+))?(-precomposed)?\.png").unwrap();
    let caps = re.captures(&path).ok_or("erorr".to_owned())?;

    let width: u32 = caps.get(2).map_or("60", |m| m.as_str()).parse().unwrap();
    let height: u32 = caps.get(3).map_or("60", |m| m.as_str()).parse().unwrap();
    Ok(Icon { width, height })
}

async fn load_source_icon(env: &Env) -> Result<DynamicImage> {
    let kv = worker::kv::KvStore::from_this(&env, "__STATIC_CONTENT")?;
    let source = kv.get("icon.jpg").bytes().await?.ok_or("erorr")?;

    let img = image::load_from_memory_with_format(&source, image::ImageFormat::Jpeg)
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(img)
}

fn generate_icon(icon: &Icon, source: &DynamicImage) -> DynamicImage {
    source.resize(
        icon.width,
        icon.height,
        image::imageops::FilterType::Triangle,
    )
}

fn make_response(icon_img: &DynamicImage) -> Result<Response> {
    let mut buf: Vec<u8> = Vec::new();
    icon_img
        .write_to(&mut Cursor::new(&mut buf), ImageOutputFormat::Png)
        .map_err(|e| Error::from(e.to_string()))?;

    let mut response = Response::from_bytes(buf)?;
    response.headers_mut().set("content-type", "image/png")?;
    Ok(response)
}
