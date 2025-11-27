use reqwest::{self, Client, header::{ACCEPT, HeaderMap, HeaderValue}};
use serde::{Serialize, Deserialize};
use slint::SharedString;

slint::include_modules!(); 

const UN_URL : &str = "https://data.un.org/ws/rest/";

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let ui = MainWindow::new().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/json"));
    let response = Client::new()
        .get(format!("{UN_URL}dataflow"))
        .headers(headers)
        .send()
        .await?;
    let mut output_txt = String::new();
    let mut depth = 0usize;
    for c in response.text().await?.chars() {
        if c == '{' {
            depth += 1;
            output_txt.push_str(&format!("\n{}{c}\n{}", vec!["    "; depth].join(""), vec!["    "; depth].join("")));
        } else if c == '}' {
            depth -= 1;
            output_txt.push_str(&format!("\n{}{c}\n", vec!["    "; depth].join("")));
        } else if c == ',' {
            output_txt.push_str(&format!("{c}\n{}", vec!["    "; depth].join("")));
        } else {
            output_txt.push_str(&format!("{c}"));
        }
    }
    ui.set_json_data(SharedString::from(output_txt));
    ui.run().unwrap();

    Ok(())
}