use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::api::types::GraphQLResponse;
use crate::error::LinError;

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

pub struct LinearClient {
    client: Client,
    token: String,
}

impl LinearClient {
    pub fn new(token: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
        }
    }

    pub async fn execute<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, LinError> {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({})),
        });

        let response = self
            .client
            .post(LINEAR_API_URL)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LinError::ApiError(format!("HTTP {status}: {text}")));
        }

        let text = response.text().await?;
        let gql_response: GraphQLResponse<T> = serde_json::from_str(&text)
            .map_err(|e| LinError::ApiError(format!("Failed to decode response: {e}")))?;

        if let Some(errors) = gql_response.errors {
            let messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(LinError::GraphQLErrors(messages));
        }

        gql_response
            .data
            .ok_or_else(|| LinError::ApiError("No data in response".to_string()))
    }
}
