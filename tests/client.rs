use std::ops::Deref;

use reqwest::{Response, header};
use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(unused)]
mod test {

    use super::*;

    #[tokio::test]
    async fn client() -> Result<()> {
        let mut client = Client::new();

        register(&client).await?;
        login(&mut client).await?;
        list(&client).await?;
        create(&client).await?;
        list(&client).await?;
        delete(&client).await?;
        list(&client).await?;

        Ok(())
    }

    async fn hello(client: &Client) -> Result<()> {
        let response = client.send(client.get("http://127.0.0.1:3000/")).await?;

        println!("\n\n=== Response for GET {} ===", response.url());
        print(client, response).await?;

        Ok(())
    }

    async fn register(client: &Client) -> Result<()> {
        let response = client
            .send(
                client
                    .post("http://127.0.0.1:3000/register")
                    .header("Content-Type", "application/json")
                    .body(r#"{"username":"william","password":"040720"}"#),
            )
            .await?;

        println!("\n\n=== Response for post {} ===", response.url());
        print(client, response).await?;

        Ok(())
    }

    async fn login(client: &mut Client) -> Result<()> {
        let response = client
            .send(
                client
                    .post("http://127.0.0.1:3000/login")
                    .header("Content-Type", "application/json")
                    .body(r#"{"username":"william","password":"040720"}"#),
            )
            .await?;

        let headers = response.headers().clone();

        println!("\n\n=== Response for POST {} ===", response.url());
        let body = print(client, response).await?;

        let body = serde_json::from_str::<AuthBody>(&body)?;

        client.cookies(headers.get("set-cookie").unwrap().to_str()?.into());
        client.access_token(body.access_token);

        Ok(())
    }

    async fn list(client: &Client) -> Result<()> {
        let response = client
            .send(client.get("http://127.0.0.1:3000/ticket"))
            .await?;

        println!("\n\n=== Response for GET {} ===", response.url());
        print(client, response).await?;

        Ok(())
    }

    async fn create(client: &Client) -> Result<()> {
        let response = client
            .send(
                client
                    .post("http://127.0.0.1:3000/ticket")
                    .header("Content-Type", "application/json")
                    .body(r#"{"title":"william"}"#),
            )
            .await?;

        println!("\n\n=== Response for POST {} ===", response.url());
        print(client, response).await?;

        Ok(())
    }

    async fn delete(client: &Client) -> Result<()> {
        let response = client
            .send(client.delete("http://127.0.0.1:3000/ticket?id=1"))
            .await?;

        println!("\n\n=== Response for POST {} ===", response.url());
        print(client, response).await?;

        Ok(())
    }

    async fn print(client: &Client, response: Response) -> Result<String> {
        let status = response.status().to_string();
        let headers = response.headers().clone();
        let body = response.text().await?;

        println!("=> {:16}: {}", "Status", status);

        println!("=> {:16}:", "Headers");
        headers.iter().for_each(|(name, value)| {
            println!("   {:16}: \"{}\"", name, value.to_str().unwrap_or_default());
        });

        println!("=> {:16}:", "Client");
        println!("   cookies: {}", client.cookies);
        println!("   access_token: {}", client.access_token);

        let content_type = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());
        let formatted_body = content_type
            .is_some_and(|content_type| content_type.contains("application/json"))
            .then(|| {
                serde_json::from_str::<serde_json::Value>(&body)
                    .and_then(|json| serde_json::to_string_pretty(&json))
                    .unwrap_or(body.clone())
            })
            .unwrap_or(body.clone());

        println!("=> {:16}:", "Response Body");
        println!("{}", formatted_body);

        Ok(body)
    }
}

#[allow(unused)]
#[derive(Deserialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Default)]
struct Client {
    client: reqwest::Client,
    cookies: String,
    access_token: String,
}

impl Client {
    fn new() -> Self {
        Default::default()
    }

    fn cookies(&mut self, cookies: String) {
        self.cookies = cookies;
    }

    fn access_token(&mut self, access_token: String) {
        self.access_token = access_token;
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<Response> {
        request
            .header("Cookie", self.cookies.clone())
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

impl Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
