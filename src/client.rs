use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};

#[derive(Debug)]
struct RawClient {
    client: reqwest::Client,
}

impl RawClient {
    pub fn new() -> Result<Self> {
        // The client currently only accepts application/json
        // The darwin-v7 documentation states that json is one
        // of the accepted content though json is the only
        // documented type see:
        // https://docs.v7labs.com/v1.0/reference/darwin-json
        // https://docs.v7labs.com/reference/darwin-json
        let content_type = "application/json";

        // Generate the headers for the http calls
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static(content_type));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

        // Build a reqwest client for use by the V1 and V2 darwin clients
        let client: reqwest::Client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }

    async fn get(&self, address: &str, api_key: &str) -> Result<reqwest::Response, reqwest::Error> {
        // Construct endpoint
        let api_key = format!("ApiKey {}", api_key);

        self.client
            .get(address)
            .header(AUTHORIZATION, api_key)
            .send()
            .await
    }

    async fn post<S: serde::Serialize + ?Sized>(
        &self,
        address: &str,
        api_key: &str,
        data: &S,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let api_key = format!("ApiKey {}", api_key);

        self.client
            .post(address)
            .header(AUTHORIZATION, api_key)
            .json(data)
            .send()
            .await
    }
}

#[derive(Debug)]
pub struct V7Client {
    api_endpoint: String,
    api_key: String,
    team: String,
    client: RawClient,
}

impl V7Client {
    pub fn new(api_endpoint: String, api_key: String, team: String) -> Result<Self> {
        let client = RawClient::new()?;

        Ok(V7Client {
            api_endpoint,
            api_key,
            team,
            client,
        })
    }

    pub fn from_config(config: &Config, team: Option<&String>) -> Result<Self> {
        // The base endpoint
        let api_endpoint = config.api_endpoint().to_string();

        // The team if not provided use the default
        let client_team = team.unwrap_or(config.default_team()).to_string();

        // Get the api key for the default team
        let api_key = &config
            .teams()
            .get(&client_team)
            .context("The requested team is not found in the config")?
            .api_key()
            .as_ref()
            .context("Api key not found in configuration")?;

        Self::new(api_endpoint, api_key.to_string(), client_team)
    }

    pub fn api_endpoint(&self) -> &str {
        &self.api_endpoint
    }

    pub fn team(&self) -> &String {
        &self.team
    }

    pub async fn get(&self, endpoint: &str) -> Result<reqwest::Response, reqwest::Error> {
        let endpoint = format!("{}/{}", self.api_endpoint, endpoint);
        self.client.get(&endpoint, &self.api_key).await
    }

    pub async fn post<S: serde::Serialize + ?Sized>(
        &self,
        endpoint: &str,
        data: &S,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let endpoint = format!("{}/{}", self.api_endpoint, endpoint);
        self.client.post(&endpoint, &self.api_key, data).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::team::Team;

    fn basic_config() -> (String, Config, Team) {
        // Setup basic properties
        let slug = "test-team".to_string();
        let base_url = "http://some-url.com".to_string();
        let api_endpoint = "http://some-url.com/api".to_string();
        let api_key = "api-key".to_string();

        // Create the team HashMap
        let test_team = Team::new(slug.clone(), None, Some(api_key));
        let mut team_map = HashMap::new();
        team_map.insert(slug.clone(), test_team.clone());

        // Create a team missing an API
        let no_api = "team-noapi".to_string();
        team_map.insert(no_api.clone(), Team::new(no_api, None, None));

        // Finally create the config
        let test_config = Config::new(
            base_url.clone(),
            api_endpoint.clone(),
            slug.clone(),
            team_map,
        );

        (api_endpoint, test_config, test_team)
    }

    #[test]
    fn test_client_from_config() {
        let (api_endpoint, test_config, test_team) = basic_config();
        let client = V7Client::from_config(&test_config, None).unwrap();

        assert_eq!(client.api_endpoint(), api_endpoint);
        assert_eq!(client.team(), test_team.slug());
    }

    #[test]
    fn test_client_wrong_team() {
        let (_api_endpoint, test_config, _test_team) = basic_config();
        V7Client::from_config(&test_config, Some(&"team-kevin".to_string()))
            .expect_err("The requested team is not found in the config");
    }

    #[test]
    fn test_client_missing_apikey() {
        let (_api_endpoint, test_config, _test_team) = basic_config();
        V7Client::from_config(&test_config, Some(&"team-noapi".to_string()))
            .expect_err("Api key not found in configuration");
    }

    #[tokio::test]
    async fn test_basic_get_call() {
        // Setup the mock endpoint
        let mock_server = MockServer::start().await;

        let api_key = "api-key-1234".to_string();

        Mock::given(method("GET"))
            .and(path("/status"))
            .and(header("accept", "application/json"))
            .and(header("content-type", "application/json"))
            .and(header(
                "Authorization",
                format!("ApiKey {}", api_key).as_str(),
            ))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        // Setup the client
        let client = V7Client::new(
            mock_server.uri().to_string(),
            api_key.clone(),
            "some-team".to_string(),
        )
        .unwrap();

        let status = client.get("status").await.unwrap().status();
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_basic_post_call() {
        // Setup the mock endpoint
        let mock_server = MockServer::start().await;

        let api_key = "api-key-1234".to_string();
        let payload = serde_json::json!({"id": "12345"});

        Mock::given(method("POST"))
            .and(path("/testpost"))
            .and(header("accept", "application/json"))
            .and(header("content-type", "application/json"))
            .and(header(
                "Authorization",
                format!("ApiKey {}", api_key).as_str(),
            ))
            .and(body_json(payload))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        // Setup the client
        let client = V7Client::new(
            mock_server.uri().to_string(),
            api_key.clone(),
            "some-team".to_string(),
        )
        .unwrap();

        let status = client
            .post("testpost", &serde_json::json!({"id": "12345"}))
            .await
            .unwrap()
            .status();
        assert_eq!(status, 200);
    }
}
