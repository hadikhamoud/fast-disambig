use http::{Request, Response};

struct Resource {
    name: String,
    uri: String,
    license: String,
    description: String,
    size: f64,
}

fn download_resource(resource: Resource) {
    let mut request = Request::builder()
        .uri(resource.uri)
        .header("User-Agent", "fast-disambig/1.0")
        .method("GET")
        .body(())
        .unwrap();
}
