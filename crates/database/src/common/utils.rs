use url::Url;

pub fn extract_domain(url: &Url) -> String {
    let mut port = String::new();
    if let Some(port_) = url.port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", url.host_str().expect("has domain"))
}

pub fn http_protocol_str() -> &'static str {
    if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    }
}
