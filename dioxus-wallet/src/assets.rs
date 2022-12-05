use dioxus_desktop::wry::http::{Request, Response, StatusCode};
use dioxus_desktop::wry::Result;
use include_dir::{include_dir, Dir};

pub static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

macro_rules! css {
    ($file:literal) => {
        crate::assets::ASSETS
            .get_file(concat!("css/", $file, ".css"))
            .unwrap()
            .contents_utf8()
            .unwrap()
    };
}

pub fn asset_handler(request: &Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
    let path = request.uri().to_string().replace("asset://", "");
    let mime = match path.rsplit_once('.') {
        Some((_, "css")) => "text/css",
        Some((_, "png")) => "image/png",
        _ => "application/octet-stream",
    };
    let file = if let Some(file) = ASSETS.get_file(path) {
        file
    } else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"Not Found".to_vec())
            .map_err(From::from);
    };
    Response::builder()
        .header("Content-Type", mime)
        .body(file.contents().to_vec())
        .map_err(From::from)
}
