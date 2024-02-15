#[cfg(feature = "ssr")]
pub fn extract_domain<T>(url: &activitypub_federation::fetch::object_id::ObjectId<T>) -> String
where
    T: activitypub_federation::traits::Object + Send + 'static,
    for<'de2> <T as activitypub_federation::traits::Object>::Kind: serde::Deserialize<'de2>,
{
    let mut port = String::new();
    if let Some(port_) = url.inner().port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", url.inner().host_str().unwrap())
}

#[cfg(not(feature = "ssr"))]
pub fn extract_domain(url: &String) -> String {
    let url = url::Url::parse(url).unwrap();
    let mut port = String::new();
    if let Some(port_) = url.port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", url.host_str().unwrap())
}

pub fn http_protocol_str() -> &'static str {
    if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    }
}
