use hyper::{
    body::Incoming as IncomingBody, Request as HyperRequest
};
use hyper_util::rt::tokio::TokioIo;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Bytes};
use hyper::{Result as HyperResult, Response as HyperResponse};
use std::net::SocketAddr;



use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};
use hyper::client::conn::http1::Builder;
use tokio::net::TcpStream;


pub(crate) async fn try_proxy_request(req: HyperRequest<hyper::body::Incoming>) -> Result<Option<HyperResponse<Full<Bytes>>>, hyper::Error> {
    if hyper::Method::CONNECT == req.method() {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            eprintln!("server io error: {}", e);
                        };
                    }
                    Err(e) => eprintln!("upgrade error: {}", e),
                }
            });

            Ok(Some(hyper::Response::new(empty())))
        } else {
            eprintln!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = hyper::Response::new(full("CONNECT must be sent to a socket address"));
            *resp.status_mut() = hyper::http::StatusCode::BAD_REQUEST;

            Ok(Some(resp))
        }
    } else if false {
        let host = req.uri().host().expect("uri has no host");
        let port = req.uri().port_u16().unwrap_or(80);

        let stream = TcpStream::connect((host, port)).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;
        Ok(Some(resp.map(|b| b.boxed())))
    } else {
        Ok(None)
    }

}

fn host_addr(uri: &hyper::http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}


fn full<T: Into<Bytes>>(chunk: T) -> Full<Bytes >{
    Full::new(chunk.into())
}

fn empty() -> Full<Bytes> {
    Full::<Bytes>::new(Bytes::new())
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    // Connect to remote server
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    // Proxying data
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    // Print message when done
    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}
