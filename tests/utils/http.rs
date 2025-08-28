use curl::easy::Easy;
use futures::channel::oneshot;
use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync>;

/// Perform an HTTP GET request using libcurl in a background thread.
///
/// This helper wraps the blocking `curl` API into an async function by spawning
/// a new thread and returning the result through a oneshot channel.
///
/// # Arguments
/// * `uri`   – The target URL to fetch.
/// * `proxy` – Optional proxy URL, e.g. `Some("http://127.0.0.1:8080")`.
///
/// # Returns
/// * `Ok((status_code, body))` where:
///   - `status_code` is the HTTP response status as `u16`.
///   - `body` is the full response body as a `String`.
///
/// # Errors
/// Returns an error if curl fails to configure, connect, or perform the request.
/// The error type is boxed (`Box<dyn Error + Send + Sync>`).
///
/// # Notes
/// * TLS verification is disabled (`ssl_verify_peer/host(false)`) for testing only.
///   Replace with proper CA setup for production use.
/// * Follows redirects automatically.
/// * Runs synchronously in a spawned thread; multiple calls will spawn multiple threads.
///
/// # Examples
/// ```no_run
/// use mycrate::utils::http::get;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let (status, body) = get("https://httpbin.org/ip", None).await?;
/// println!("Status: {status}, Body: {body}");
///
/// let (status, body) = get("https://httpbin.org/ip", Some("http://127.0.0.1:8080")).await?;
/// println!("Via proxy → {status}, {body}");
/// # Ok(())
/// # }
/// ```
pub async fn get(uri: &str, proxy: Option<&str>) -> Result<(u16, String), BoxError> {
    let uri = uri.to_string();
    let proxy = proxy.map(|p| p.to_string());

    let (tx, rx) = oneshot::channel::<Result<(u16, String), BoxError>>();

    std::thread::spawn(move || {
        let mut easy = Easy::new();
        let mut buf = Vec::new();

        if let Err(e) = easy.url(&uri) {
            let _ = tx.send(Err(Box::new(e) as BoxError));
            return;
        }
        if let Some(p) = proxy.as_deref() {
            if let Err(e) = easy.proxy(p) {
                let _ = tx.send(Err(Box::new(e) as BoxError));
                return;
            }
        }

        // tests only
        let _ = easy.ssl_verify_peer(false);
        let _ = easy.ssl_verify_host(false);
        let _ = easy.follow_location(true);

        let mut transfer = easy.transfer();
        if let Err(e) = transfer.write_function(|chunk| {
            buf.extend_from_slice(chunk);
            Ok(chunk.len())
        }) {
            let _ = tx.send(Err(Box::new(e) as BoxError));
            return;
        }
        if let Err(e) = transfer.perform() {
            let _ = tx.send(Err(Box::new(e) as BoxError));
            return;
        }
        drop(transfer);

        let status = easy.response_code().unwrap_or(0) as u16;
        let body = String::from_utf8_lossy(&buf).to_string();
        let _ = tx.send(Ok((status, body)));
    });

    rx.await.unwrap()
}
