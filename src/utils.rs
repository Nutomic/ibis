use rand::{distributions::Alphanumeric, thread_rng, Rng};
use url::{ParseError, Url};

pub fn generate_activity_id(domain: &Url) -> Result<Url, ParseError> {
    let port = domain.port().unwrap();
    let domain = domain.domain().unwrap();
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!("http://{}:{}/objects/{}", domain, port, id))
}
