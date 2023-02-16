#![deny(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![allow(clippy::missing_errors_doc)]

mod errors;
mod highlight;
mod io;
mod params;

use crate::{
    errors::{InternalServerError, NotFound},
    highlight::highlight,
    io::PasteDiskRepository,
    params::{HostHeader, IsPlaintextRequest},
};

use actix_web::{
    http::header,
    web::{self, Bytes, Data, FormConfig, PayloadConfig},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::{Html as AskamaHtml, MarkupDisplay, Template};
use log::{error, info};
use once_cell::sync::Lazy;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
};
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

pub trait PasteRepository {
    fn create(&self, id: &str, content: Bytes) -> Result<(), std::io::Error>;
    fn read(&self, id: &str) -> Option<Bytes>;
    fn exists(&self, id: &str) -> bool;
}

#[derive(argh::FromArgs, Clone)]
/// a pastebin.
pub struct BinArgs {
    /// socket address to bind to (default: 127.0.0.1:8820)
    #[argh(
        positional,
        default = "SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8820)"
    )]
    bind_addr: SocketAddr,
    /// maximum paste size in bytes (default. 32kB)
    #[argh(option, default = "32 * 1024")]
    max_paste_size: usize,
    /// location of paste storage directory
    #[argh(option, default = "String::from(\"/srv/pastes\")")]
    paste_dir: String,
}

/// Generates a random id, avoiding confusable characters
fn generate_id() -> String {
    let valid_chars = "abcdefghjkmnpqrstuvwxyzABCDEFGHJKMNPQRSTUVWXYZ"; // Avoids i, l, and o
    let chars = valid_chars.chars().collect::<Vec<char>>();
    let range = Uniform::from(0..valid_chars.len());
    thread_rng()
        .sample_iter(range)
        .take(12)
        .map(|x| chars[x])
        .collect()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init();

    let args: BinArgs = argh::from_env();
    let paste_repository = Data::new(PasteDiskRepository {});

    let path = Path::new(&args.paste_dir);
    if !path.is_dir() {
        eprintln!("The paste storage directory does not exist.");
        std::process::exit(1);
    }

    let server = HttpServer::new({
        let args = args.clone();

        move || {
            App::new()
                .app_data(paste_repository.clone())
                .app_data(PayloadConfig::default().limit(args.max_paste_size))
                .app_data(FormConfig::default().limit(args.max_paste_size))
                .wrap(actix_web::middleware::Compress::default())
                .route("/", web::get().to(index))
                .route("/", web::post().to(submit::<PasteDiskRepository>))
                .route("/", web::put().to(submit_raw::<PasteDiskRepository>))
                .route("/", web::head().to(HttpResponse::MethodNotAllowed))
                .route("/highlight.css", web::get().to(highlight_css))
                .route("/{paste}", web::get().to(show_paste::<PasteDiskRepository>))
                .route("/{paste}", web::head().to(HttpResponse::MethodNotAllowed))
                .default_service(web::to(|req: HttpRequest| async move {
                    error!("Couldn't find resource {}", req.uri());
                    HttpResponse::from_error(NotFound)
                }))
        }
    });

    info!("Listening on http://{}", args.bind_addr);

    server.bind(args.bind_addr)?.run().await
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

async fn index(req: HttpRequest) -> Result<HttpResponse, Error> {
    render_template(&req, &Index)
}

#[derive(serde::Deserialize)]
struct IndexForm {
    val: Bytes,
}

async fn submit<T: PasteRepository>(
    input: web::Form<IndexForm>,
    repository: Data<T>,
) -> impl Responder {
    let mut id = generate_id();
    while repository.exists(&id) {
        id = generate_id();
    }
    let uri = format!("/{}", &id);
    repository
        .create(&id, input.into_inner().val)
        .expect("Failed creating file");
    HttpResponse::Found()
        .append_header((header::LOCATION, uri))
        .finish()
}

async fn submit_raw<T: PasteRepository>(
    data: Bytes,
    host: HostHeader,
    repository: Data<T>,
) -> Result<String, Error> {
    let mut id = generate_id();
    while repository.exists(&id) {
        id = generate_id();
    }
    let uri = if let Some(Ok(host)) = host.0.as_ref().map(|v| std::str::from_utf8(v.as_bytes())) {
        format!("https://{host}/{id}")
    } else {
        format!("/{id}")
    };

    repository.create(&id, data).expect("Failed creating file");
    Ok(uri)
}

#[derive(Template)]
#[template(path = "paste.html")]
struct ShowPaste<'a> {
    content: MarkupDisplay<AskamaHtml, Cow<'a, String>>,
}

async fn show_paste<T: PasteRepository>(
    req: HttpRequest,
    key: web::Path<String>,
    plaintext: IsPlaintextRequest,
    repository: Data<T>,
) -> Result<HttpResponse, Error> {
    let mut splitter = key.splitn(2, '.');
    let key = splitter.next().unwrap();
    let ext = splitter.next();

    let entry = repository.read(key).ok_or(NotFound)?;

    if *plaintext {
        Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(entry))
    } else {
        let data = std::str::from_utf8(entry.as_ref())?;

        let code_highlighted = match ext {
            Some(extension) => match highlight(data, extension) {
                Some(html) => html,
                None => return Err(NotFound.into()),
            },
            None => htmlescape::encode_minimal(data),
        };

        // Add <code> tags to enable line numbering with CSS
        let html = format!(
            "<code>{}</code>",
            code_highlighted.replace('\n', "</code><code>")
        );

        let content = MarkupDisplay::new_safe(Cow::Borrowed(&html), AskamaHtml);

        render_template(&req, &ShowPaste { content })
    }
}

async fn highlight_css() -> HttpResponse {
    static CSS: Lazy<Bytes> = Lazy::new(|| {
        highlight::BAT_ASSETS.with(|s| {
            Bytes::from(css_for_theme_with_class_style(
                s.get_theme("OneHalfDark"),
                ClassStyle::Spaced,
            ))
        })
    });

    HttpResponse::Ok()
        .content_type("text/css")
        .body(CSS.clone())
}

fn render_template<T: Template>(req: &HttpRequest, template: &T) -> Result<HttpResponse, Error> {
    match template.render() {
        Ok(html) => Ok(HttpResponse::Ok().content_type("text/html").body(html)),
        Err(e) => {
            error!("Error while rendering template for {}: {}", req.uri(), e);
            Err(InternalServerError(Box::new(e)).into())
        }
    }
}
