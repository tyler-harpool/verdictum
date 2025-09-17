use spin_sdk::http::{IntoResponse, Params, Request, Response, ResponseBuilder};
use std::sync::Arc;
use url::Url;
use utoipa::openapi::ServerBuilder;
use utoipa::OpenApi;
pub fn get_openapi_description(req: Request, _: Params) -> anyhow::Result<impl IntoResponse> {
    let mut openapi_description = OpenApiDocs::openapi();
    let (url, description) = get_server_info(&req);
    openapi_description.servers = Some(vec![ServerBuilder::new()
        .url(url)
        .description(Some(description))
        .build()]);

    Ok(ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(openapi_description.to_pretty_json()?)
        .build())
}

fn get_server_info(req: &Request) -> (String, String) {
    match is_local_spin_runtime(&req) {
        true => {
            let full_url = req
                .header("spin-full-url")
                .expect("spin-full-url should be set when running api with spin CLI")
                .as_str()
                .expect("spin-full-url shall not be empty when running api with spin");

            let u = Url::parse(full_url).expect("spin-full-url should be a valid url");
            (
                format!(
                    "{}://{}/",
                    u.scheme(),
                    format!(
                        "{}:{}",
                        u.host_str().expect("spin-full-url should have host"),
                        u.port().expect("spin-full-url should have port")
                    )
                ),
                String::from("Local Development Server"),
            )
        }
        false => {
            let host = req
                .header("x-forwarded-host")
                .expect("x-forwarded-host should be set via FWF")
                .as_str()
                .expect("x-forwarded-host shall not be empty on FWF")
                .to_string();
            (
                format!("https://{host}/"),
                String::from("Production Server"),
            )
        }
    }
}

fn is_local_spin_runtime(req: &Request) -> bool {
    req.header("spin-client-addr").is_some()
}
pub fn render_openapi_docs_ui(req: Request, _p: Params) -> anyhow::Result<impl IntoResponse> {
    let mut path = req
        .header("spin-path-info")
        .expect("spin-path-info is not present")
        .as_str()
        .unwrap_or("/")
        .to_string();

    path = path.replace("/docs/", "");

    let config = Arc::new(utoipa_swagger_ui::Config::from("openapi-description.json"));

    Ok(match utoipa_swagger_ui::serve(path.as_ref(), config) {
        Ok(swagger_file) => swagger_file
            .map(|file| {
                ResponseBuilder::new(200)
                    .header("content-type", file.content_type)
                    .body(file.bytes.to_vec())
                    .build()
            })
            .unwrap_or_else(|| Response::new(404, "Not Found")),
        Err(_) => Response::new(500, "Internal Server Error"),
    })
}

#[derive(OpenApi)]
#[openapi(
  info(
    title = "ToDo API",
    description = "An awesome ToDo API",
    terms_of_service = include_str!("../../fake_terms_of_service.txt"),
    contact(
      name = "Fermyon Engineering", 
      email = "engineering@fermyon.com", 
      url = "https://www.fermyon.com"
    )
  ),
  tags(
  	(name = "todos", description = "ToDo API Endpoints")
  ),
  paths(
  	crate::handlers::todo::get_by_id,
  )
)]
struct OpenApiDocs {}
