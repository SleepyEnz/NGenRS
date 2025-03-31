pub async fn http_get_async(url: &str) -> Option<String> {
    match reqwest::get(url).await {
        Ok(response) => {
            if response.status().is_success() {
                response.text().await.ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}