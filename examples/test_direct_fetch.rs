use hyper::Request;
use http_body_util::{BodyExt, Empty};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=======================================================");
    println!("Direct Fetch Test - Using Rigging's Network Stack");
    println!("=======================================================\n");

    // Create HTTP client with Unix connector (same as Harbor uses internally)
    println!("[1] Setting up UnixConnector...");

    // Import Rigging's Unix connector
    use rigging::unix_connector::UnixConnector;
    let connector = UnixConnector::new("/tmp/hello-harbor.sock");

    type EmptyBody = Empty<&'static [u8]>;
    let client = Client::builder(TokioExecutor::new())
        .build::<_, EmptyBody>(connector);

    println!("[2] Creating HTTP request to http://localhost/...");
    let req = Request::builder()
        .method("GET")
        .uri("http://localhost/")
        .header("Host", "localhost")
        .header("Connection", "close")
        .body(Empty::<&[u8]>::new())?;

    println!("[3] Sending request over Unix socket...\n");
    let res = client.request(req).await?;

    println!("========== RESPONSE ==========");
    println!("Status: {}", res.status());
    println!("Headers:");
    for (name, value) in res.headers() {
        println!("  {}: {:?}", name, value);
    }
    println!("\n========== BODY ==========");

    let body_bytes = res.into_body().collect().await?.to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);

    println!("{}", body_str);

    println!("\n========== SUMMARY ==========");
    println!("Received {} bytes", body_bytes.len());
    println!("Body is valid HTML: {}", body_str.starts_with("<!DOCTYPE html>") || body_str.starts_with("<html"));

    Ok(())
}
