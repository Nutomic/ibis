use idna::domain_to_unicode;
use url::Url;

pub fn extract_domain(url: &Url) -> String {
    let mut port = String::new();
    if let Some(port_) = url.port() {
        port = format!(":{port_}");
    }
    let domain = url.host_str().expect("has domain");

    format!("{}{port}", domain_to_unicode(domain).0)
}

pub fn http_protocol_str() -> &'static str {
    if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    }
}
