use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use reqwest::Client;
use reqwest::header::HeaderMap;
use futures::StreamExt;

pub struct HttpClient {
    client: Client,
}

pub struct HttpResponse {
    pub status: reqwest::StatusCode,
    pub headers: HeaderMap,
    pub body: Option<String>,
}

impl HttpClient {
    pub fn new(ca_cert_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(false);

        if let Some(cert_path) = ca_cert_path {
            let cert = std::fs::read(cert_path)?;
            builder = builder.add_root_certificate(
                reqwest::Certificate::from_pem(&cert)?
            );
        }

        Ok(Self {
            client: builder.build()?
        })
    }

    async fn execute_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let response = request.send().await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await.ok();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    pub async fn get(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,  // Changed from &str to String
        body: Option<&str>,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let mut request = self.client.get(url);

        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(&key, &value);  // Borrow the Strings here
            }
        }

        if let Some(body_content) = body {
            request = request.body(body_content.to_string());
        }

        self.execute_request(request).await
    }

    fn convert_string_map_to_json(
        params: Option<HashMap<String, String>>
    ) -> Option<HashMap<String, Value>> {
        params.map(|map| {
            map.into_iter()
                .filter_map(|(k, v)| {
                    serde_json::from_str::<Value>(&v)
                        .map(|val| (k, val))
                        .ok()
                })
                .collect()
        })
    }

    pub async fn post(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&str>,
        params: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let mut request = self.client.post(url);

        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(&key, &value);
            }
        }

        if let Some(json_map) = Self::convert_string_map_to_json(params) {
            request = request.json(&json_map);
        } else if let Some(body_content) = body {
            request = request.body(body_content.to_string());
        }

        self.execute_request(request).await
    }

    pub async fn download(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
        output_path: &Path,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let mut request = self.client.get(url);

        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(key, value);
            }
        }

        let response = request.send().await?;
        let status = response.status();
        let headers = response.headers().clone();

        // Stream the response body to file
        let mut file = tokio::fs::File::create(output_path).await?;
        let mut stream = response.bytes_stream(); // Now this will work

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::copy(&mut chunk.as_ref(), &mut file).await?;
        }

        Ok(HttpResponse {
            status,
            headers,
            body: None,
        })
    }
}