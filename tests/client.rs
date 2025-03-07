use reqwest::Client;
use reqwest::Response;
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

mod test {

    use super::*;

    async fn print(response: Response) -> Result<(), Box<dyn std::error::Error>> {
        let headers = response.headers().clone();
        let body = response.text().await?;

        println!("=> {:16}:", "Headers");
        headers.iter().for_each(|(name, value)| {
            println!("   {}: \"{}\"", name, value.to_str().unwrap_or_default());
        });

        let content_type = headers
            .get(reqwest::header::CONTENT_TYPE)
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

    #[tokio::test]
    async fn hello() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client.get("http://127.0.0.1:3000/").send().await?;

        print!("{:?}", response.text().await?);

        Ok(())
    }

    #[tokio::test]
    async fn register() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client
            .post("http://127.0.0.1:3000/register")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        print!("{:?}", response.text().await?);

        Ok(())
    }

    #[tokio::test]
    async fn login() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client
            .post("http://127.0.0.1:3000/login")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        print!("{:?}", response.text().await?);

        Ok(())
    }

    async fn auth() -> Result<(String, String), Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client
            .post("http://127.0.0.1:3000/login")
            .header("Content-Type", "application/json")
            .body(r#"{"username":"william","password":"040720"}"#)
            .send()
            .await?;

        let headers = response.headers().clone();
        let body = response.json::<AuthBody>().await?;

        let cookie = headers.get("set-cookie").unwrap().to_str()?.to_string();

        Ok((cookie, body.access_token))
    }

    #[tokio::test]
    async fn list() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();

        let (cookie, access_token) = auth().await?;

        let response = client
            .get("http://127.0.0.1:3000/ticket")
            .header("Cookie", cookie)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        print(response).await?;

        Ok(())
    }
}
