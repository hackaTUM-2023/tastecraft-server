use clap::{Error, Parser};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::config::Config;
use crate::models::recipes::Recipe;
use anyhow::Result;


#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    text: String,
    finish_log_prob: f64,
    log_prob: f64,
}

const SYSTEM_PROMPT: &str = "Given a recipe in JSON format and a list of preferred
ingredients, adjust the recipe to match the preferences and return the adjusted recipe
in JSON format. The json format should be as follows:
{
    \"title\": \"Recipe Title\",
    \"description\": \"Recipe Description\",
    \"instructions\": \"Recipe Instructions\",
    \"preptime\": 30,
    \"difficulty\": 1,
    \"ingredients\": [
        \"ingredient1\",
        \"ingredient2\",
        \"ingredient3\"
    ],
    \"isoriginal\": false
} ";

pub async fn send_request(recipe: &Recipe, preferences: &[&str]) -> Result<Recipe> {
    let _config = Config::parse();

    let url = Url::parse("https://api.openai.com/v1/chat/completions").expect("Invalid URL");
    let recipe_string = serde_json::to_string(&recipe).expect("Failed to serialize recipe");
    let preferences_string = preferences.join(", ");
    let prompt = format!("Recipe: {recipe_string}, preferences: {preferences_string}");
    let request = json!({
        "model": "gpt-4-1106-preview",
    "messages": [
      {
        "role": "system",
        "content": SYSTEM_PROMPT
      },
      {
        "role": "user",
        "content": prompt
      }
    ],
    "n": 1,
    "response_format": {
        "type": "json_object"
    },
    "temperature": 1
    });

    let client = Client::new();
    let mut req = client.post(url);
    let req = req.header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", _config.openai_key))
        .body(request.to_string());

    let response = req.send().await.expect("Failed to send request");
    let response_text = response.text().await.expect("Failed to parse response");

    let v: Value = serde_json::from_str(response_text.as_str()).expect("Failed to parse JSON");
    let res: Recipe = serde_json::from_str(v["choices"][0]["message"]["content"].as_str().unwrap()).expect("Failed to parse JSON");
    Ok(res)
}