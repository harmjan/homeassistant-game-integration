use reqwest;
use serde::Deserialize;
use thiserror::Error;

/// An error that can occur while executing an action
#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Failed to execute webhook")]
    WebhookError(#[from] reqwest::Error),
}

/// The configuration for an action
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    #[serde(rename = "webhook")]
    Webhook { url: String },
}

impl Config {
    /// Execute the action
    pub fn execute(&self) -> Result<(), ActionError> {
        match self {
            Config::Webhook { url } => {
                let client = reqwest::blocking::Client::new();
                client.post(url).send()?.error_for_status()?;
            }
        };

        Ok(())
    }
}
