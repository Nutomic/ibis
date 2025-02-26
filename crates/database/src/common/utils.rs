use url::Url;

pub fn extract_domain(url: &Url) -> String {
    let mut port = String::new();
    if let Some(port_) = url.port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", url.host_str().expect("has domain"))
}

/*
#[cfg(not(feature = "ssr"))]
pub fn extract_domain(url: &String) -> String {
    let url = Url::parse(url).unwrap();
    let mut port = String::new();
    if let Some(port_) = url.port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", url.host_str().unwrap())
}
*/

pub fn http_protocol_str() -> &'static str {
    if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    }
}
