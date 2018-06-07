use reqwest::{self, Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use url::Url;

use commands::CommandError;
use config::Config;

const API_VERSION: &'static str = "2018-06-01";

header! { (XAPIVersion, "X-API-Version") => [String] }

#[derive(Serialize, Deserialize)]
pub struct APIError {
    pub error: String,
}

pub struct API<'a> {
    config: &'a Config,
}

impl<'a> API<'a> {
    pub fn new(config: &'a Config) -> API<'a> {
        API { config }
    }

    pub fn apps(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "apps/")
    }

    pub fn app(&self, name: &String) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "apps/").param(name)
    }

    pub fn login(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "login")
    }

    pub fn signup(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "signup")
    }

    pub fn refresh(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "refresh")
    }

    pub fn has_card(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "has-card")
    }

    pub fn tiers(&self) -> APIRequestBuilder {
        APIRequestBuilder::new(self.config, "tiers")
    }
}

pub struct APIRequestBuilder {
    client: reqwest::Client,
    route: Url,
}

impl APIRequestBuilder {
    fn new(config: &Config, path: &'static str) -> APIRequestBuilder {
        let mut client = reqwest::Client::builder();
        let mut headers = reqwest::header::Headers::new();
        headers.set(XAPIVersion(API_VERSION.to_owned()));
        if config.token != "" {
            headers.set(config.get_auth_header());
        }
        client.default_headers(headers);
        APIRequestBuilder {
            client: client.build().unwrap(),
            route: config.endpoint.join(path).unwrap(),
        }
    }

    fn get_response<R: DeserializeOwned>(
        &self,
        req: &mut reqwest::RequestBuilder,
    ) -> Result<R, CommandError> {
        let mut res = req.send()?;
        match res.status() {
            StatusCode::Ok => res.json().map_err(|err| {
                CommandError::with_message(format!("Failed to parse response: {}", err))
            }),
            _ => {
                let APIError { error } = res
                    .json()
                    .map_err(|_| CommandError::with_message("Failed to parse response."))?;
                Err(CommandError::with_message(error))
            }
        }
    }

    pub fn param(self, p: &str) -> APIRequestBuilder {
        APIRequestBuilder {
            client: self.client,
            route: self.route.join(&format!("{}/", p)).unwrap(),
        }
    }

    pub fn get<R: DeserializeOwned>(&self) -> Result<R, CommandError> {
        let mut req = self.client.get(self.route.clone());
        self.get_response(&mut req)
    }

    pub fn post<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        json: &T,
    ) -> Result<R, CommandError> {
        let mut req = self.client.post(self.route.clone());
        self.get_response(req.json(json))
    }

    pub fn delete<R: DeserializeOwned>(&self) -> Result<R, CommandError> {
        let mut req = self.client.delete(self.route.clone());
        self.get_response(&mut req)
    }

    pub fn request(&self, method: Method) -> RequestBuilder {
        self.client.request(method, self.route.clone())
    }
}
