use reqwest::{Client, Response, header};
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

#[allow(unused)]
mod test {

    use super::*;

    #[tokio::test]
    async fn client() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();

        let (cookies, access_token) = auth(&client).await?;

        login(&client).await?;

        Ok(())
    }

    async fn auth(client: &Client) -> Result<(String, String), Box<dyn std::error::Error>> {
        let response = client
            .post("http://127.0.0.1:3000/login")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        let headers = response.headers().clone();
        let body = response.json::<AuthBody>().await?;

        let cookies = headers.get("set-cookie").unwrap().to_str()?.to_string();
        let access_token = body.access_token;

        Ok((cookies, access_token))
    }

    async fn hello(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let response = client.get("http://127.0.0.1:3000/").send().await?;

        println!("\n\n=== Response for GET {} ===", response.url());
        print(response, "".to_string(), "".to_string()).await?;

        Ok(())
    }

    async fn register(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .post("http://127.0.0.1:3000/register")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        println!("\n\n=== Response for post {} ===", response.url());
        print(response, "".to_string(), "".to_string()).await?;

        Ok(())
    }

    async fn login(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .post("http://127.0.0.1:3000/login")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        println!("\n\n=== Response for POST {} ===", response.url());
        print(response, "".to_string(), "".to_string()).await?;

        Ok(())
    }

    async fn list(
        client: &Client,
        cookies: String,
        access_token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .get("http://127.0.0.1:3000/ticket")
            .header("Cookie", cookies.clone())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        println!("\n\n=== Response for GET {} ===", response.url());
        print(response, cookies, access_token).await?;

        Ok(())
    }

    async fn print(
        response: Response,
        cookies: String,
        access_token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let status = response.status().to_string();
        let headers = response.headers().clone();
        let body = response.text().await?;

        println!("=> {:16}: {}", "Status", status);

        println!("=> {:16}:", "Headers");
        headers.iter().for_each(|(name, value)| {
            println!("   {:16}: \"{}\"", name, value.to_str().unwrap_or_default());
        });

        println!("=> {:16}:", "Client");
        println!("   cookies: {}", cookies);
        println!("   access_token: {}", access_token);

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
            .unwrap_or(body);

        println!("=> {:16}:", "Response Body");
        println!("{}", formatted_body);

        Ok(())
    }
}
