use std::fs;
use std::io::{self, BufWriter, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::redirect;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::cli::FeedInput;

const CHUNK_SIZE: usize = 65_536;

static BAR_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{msg} [{bar:30.cyan/dim}] {bytes}/{total_bytes}")
        .expect("hard-coded progress template is valid")
        .progress_chars("█░░")
});

static SPINNER_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .expect("hard-coded spinner template is valid")
        .tick_chars("⣷⣯⣟⡿⢿⣻⣽⣾ ")
});

const VALID_CONTENT_TYPES: &[&str] = &[
    "application/zip",
    "application/octet-stream",
    "application/x-zip-compressed",
];

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP {status}: {url}")]
    HttpStatus { status: u16, url: String },
    #[error("connection timed out after {secs}s")]
    ConnectTimeout { secs: u64 },
    #[error("read timed out after {secs}s")]
    ReadTimeout { secs: u64 },
    #[error("too many redirects (max {max})")]
    TooManyRedirects { max: usize },
    #[error("feed size ({actual}) exceeds maximum ({max})")]
    TooLarge { actual: String, max: String },
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("HTTP client error: {0}")]
    Client(String),
    #[error("cache error: {0}")]
    Cache(String),
}

#[derive(Debug)]
pub struct DownloadOptions {
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub max_redirects: usize,
    pub max_size_bytes: u64,
    pub no_cache: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 10,
            read_timeout_secs: 60,
            max_redirects: 5,
            max_size_bytes: 500 * 1024 * 1024,
            no_cache: false,
        }
    }
}

pub enum ClientResponse {
    NotModified,
    Body {
        content_type: Option<String>,
        content_length: Option<u64>,
        etag: Option<String>,
        last_modified: Option<String>,
        reader: Box<dyn Read + Send>,
    },
}

pub trait HttpClient: Send {
    /// # Errors
    /// Returns `HttpError` on network failure, HTTP error status, timeout, or redirect limit.
    fn execute(
        &self,
        url: &str,
        opts: &DownloadOptions,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<ClientResponse, HttpError>;
}

pub struct ReqwestClient;

impl ReqwestClient {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for ReqwestClient {
    fn execute(
        &self,
        url: &str,
        opts: &DownloadOptions,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<ClientResponse, HttpError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(opts.connect_timeout_secs))
            .timeout(Duration::from_secs(opts.read_timeout_secs))
            .redirect(redirect::Policy::limited(opts.max_redirects))
            .build()
            .map_err(|e| HttpError::Client(e.to_string()))?;

        let mut req = client.get(url);
        if let Some(etag) = etag {
            req = req.header("If-None-Match", etag);
        }
        if let Some(lm) = last_modified {
            req = req.header("If-Modified-Since", lm);
        }

        let resp = req.send().map_err(|e| map_reqwest_error(&e, opts))?;

        let status = resp.status();
        if status.as_u16() == 304 {
            return Ok(ClientResponse::NotModified);
        }
        if !status.is_success() {
            return Err(HttpError::HttpStatus {
                status: status.as_u16(),
                url: url.to_owned(),
            });
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).trim().to_owned());

        let content_length = resp.content_length();

        let etag_out = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        let last_modified_out = resp
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        Ok(ClientResponse::Body {
            content_type,
            content_length,
            etag: etag_out,
            last_modified: last_modified_out,
            reader: Box::new(resp),
        })
    }
}

fn map_reqwest_error(e: &reqwest::Error, opts: &DownloadOptions) -> HttpError {
    if e.is_redirect() {
        return HttpError::TooManyRedirects {
            max: opts.max_redirects,
        };
    }
    if e.is_connect() {
        return HttpError::ConnectTimeout {
            secs: opts.connect_timeout_secs,
        };
    }
    if e.is_timeout() {
        return HttpError::ReadTimeout {
            secs: opts.read_timeout_secs,
        };
    }
    HttpError::Client(e.to_string())
}

#[derive(Serialize, Deserialize)]
struct CacheMetadata {
    url: String,
    etag: Option<String>,
    last_modified: Option<String>,
    zip_path: PathBuf,
}

fn cache_paths(url: &str) -> Option<(PathBuf, PathBuf)> {
    let base = dirs::cache_dir()?.join("gapline").join("downloads");
    let hex = blake3::hash(url.as_bytes()).to_hex();
    let zip = base.join(format!("{hex}.zip"));
    let meta = base.join(format!("{hex}.json"));
    Some((zip, meta))
}

fn load_cache(url: &str) -> Option<CacheMetadata> {
    let (zip_path, meta_path) = cache_paths(url)?;
    if !zip_path.exists() || !meta_path.is_file() {
        return None;
    }
    let raw = fs::read_to_string(&meta_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_cache(meta: &CacheMetadata) -> Result<(), HttpError> {
    let (_, meta_path) = cache_paths(&meta.url)
        .ok_or_else(|| HttpError::Cache("could not determine cache directory".to_owned()))?;
    if let Some(parent) = meta_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(meta).map_err(|e| HttpError::Cache(e.to_string()))?;
    fs::write(&meta_path, json)?;
    Ok(())
}

fn copy_to_cache(src: &Path, dst: &Path) -> Result<(), HttpError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn fmt_bytes(n: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    const KIB: u64 = 1024;
    if n >= GIB {
        format!("{:.1} GB", n as f64 / GIB as f64)
    } else if n >= MIB {
        format!("{:.1} MB", n as f64 / MIB as f64)
    } else if n >= KIB {
        format!("{:.1} KB", n as f64 / KIB as f64)
    } else {
        format!("{n} B")
    }
}

/// Download a GTFS feed from `url` to a temporary file, with optional caching.
///
/// # Errors
/// Returns `HttpError` on network failure, HTTP error status, size limit exceeded,
/// I/O failure writing the temp file, or cache I/O errors.
pub fn download_feed(
    url: &str,
    opts: &DownloadOptions,
    client: &dyn HttpClient,
) -> Result<NamedTempFile, HttpError> {
    let cache_meta = if opts.no_cache { None } else { load_cache(url) };

    let (etag, lm) = cache_meta.as_ref().map_or((None, None), |m| {
        (m.etag.as_deref(), m.last_modified.as_deref())
    });

    let resp = client.execute(url, opts, etag, lm)?;

    match resp {
        ClientResponse::NotModified => {
            let cached = cache_meta.ok_or_else(|| {
                HttpError::Cache(
                    "server returned 304 Not Modified but no local cache exists".to_owned(),
                )
            })?;
            let tmp = tempfile::Builder::new().suffix(".zip").tempfile()?;
            fs::copy(&cached.zip_path, tmp.path())?;
            Ok(tmp)
        }
        ClientResponse::Body {
            content_type,
            content_length,
            etag: new_etag,
            last_modified: new_lm,
            mut reader,
        } => {
            if let Some(ct) = &content_type {
                let base = ct.split(';').next().unwrap_or(ct).trim();
                if !VALID_CONTENT_TYPES.contains(&base) {
                    eprintln!("warning: unexpected Content-Type '{ct}' (expected application/zip)");
                }
            }

            if let Some(len) = content_length
                && len > opts.max_size_bytes
            {
                return Err(HttpError::TooLarge {
                    actual: fmt_bytes(len),
                    max: fmt_bytes(opts.max_size_bytes),
                });
            }

            let pb = if let Some(len) = content_length {
                let pb = ProgressBar::new(len);
                pb.set_style(BAR_STYLE.clone());
                pb.set_message("Downloading feed");
                pb
            } else {
                let pb = ProgressBar::new_spinner();
                pb.set_style(SPINNER_STYLE.clone());
                pb.set_message("Downloading feed");
                pb
            };
            if std::io::stderr().is_terminal() {
                pb.enable_steady_tick(Duration::from_millis(100));
            } else {
                pb.set_draw_target(ProgressDrawTarget::hidden());
            }

            let tmp = tempfile::Builder::new().suffix(".zip").tempfile()?;
            {
                let mut writer = BufWriter::new(tmp.as_file());
                let mut buf = vec![0u8; CHUNK_SIZE];
                loop {
                    let n = reader.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    writer.write_all(&buf[..n])?;
                    pb.inc(n as u64);
                }
                writer.flush()?;
            }
            tmp.as_file().sync_all()?;
            pb.finish_and_clear();

            if !opts.no_cache
                && let Some((zip_path, _)) = cache_paths(url)
                && copy_to_cache(tmp.path(), &zip_path).is_ok()
            {
                let meta = CacheMetadata {
                    url: url.to_owned(),
                    etag: new_etag,
                    last_modified: new_lm,
                    zip_path,
                };
                let _ = save_cache(&meta);
            }

            Ok(tmp)
        }
    }
}

/// Resolve a `FeedInput` to a local `PathBuf`, downloading via HTTP if needed.
///
/// Returns the path and an optional `NamedTempFile` that the caller must keep
/// alive for the duration of feed processing (dropped = deleted).
///
/// # Errors
/// Propagates `HttpError` from `download_feed` when the input is a URL.
pub fn resolve_feed(
    input: Option<&FeedInput>,
    opts: &DownloadOptions,
) -> Result<(Option<PathBuf>, Option<NamedTempFile>), HttpError> {
    match input {
        None => Ok((None, None)),
        Some(FeedInput::Path(p)) => Ok((Some(p.clone()), None)),
        Some(FeedInput::Url(u)) => {
            let client = ReqwestClient::new();
            let tmp = download_feed(u, opts, &client)?;
            let path = tmp.path().to_owned();
            Ok((Some(path), Some(tmp)))
        }
    }
}
