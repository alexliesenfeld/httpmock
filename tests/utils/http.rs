use reqwest::blocking::Client;
use tokio::sync::oneshot;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Perform an HTTP GET request using reqwest's blocking client in a background thread.
///
/// This helper wraps the blocking `reqwest` API into an async function by spawning
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
/// Returns an error if the client fails to configure, connect, or perform the request.
///
pub async fn get(uri: &str, proxy: Option<&str>) -> Result<(u16, String), BoxError> {
    let uri = uri.to_string();
    let proxy = proxy.map(|p| p.to_string());

    let (tx, rx) = oneshot::channel::<Result<(u16, String), BoxError>>();

    std::thread::spawn(move || {
        let mut builder = Client::builder()
            .danger_accept_invalid_certs(true) // testing only
            .danger_accept_invalid_hostnames(true);

        if let Some(p) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(p).unwrap());
        }

        let client = match builder.build() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Err(Box::new(e)));
                return;
            }
        };

        let res = client.get(&uri).send();
        match res {
            Ok(r) => {
                let status = r.status().as_u16();
                match r.text() {
                    Ok(body) => {
                        let _ = tx.send(Ok((status, body)));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Box::new(e)));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Err(Box::new(e)));
            }
        }
    });

    rx.await.unwrap()
}
