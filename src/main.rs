use std::{collections::HashMap, fmt::Display, fs::File};

use reqwest::{self, Client, header::{ACCEPT, HeaderMap, HeaderValue}};
use serde::{Serialize, Deserialize};
use slint::{SharedString, PhysicalSize};
use std::io::prelude::*;
use quick_xml::{events::Event, reader::{self, Reader}};
use futures_util::StreamExt;

slint::include_modules!(); 

const EUROSTAT_URL : &str = "https://ec.europa.eu/eurostat/api/dissemination/";

struct EurostatDataset {
    name: String,
    code: String,
    datatype: String,
    start: String,
    end: String,
    size: String,
    depth: usize,
}

impl EurostatDataset {
    fn try_from(s: &str) -> Option<Self> {
        let mut fields = s.split("\t").map(|x| x.replace("\"", "").to_string());
        let title = fields.next()?;
        let tabs = title.chars().take_while(|c| c.is_whitespace()).count()/4;

        Some(EurostatDataset {
            name: title.trim().to_string(),
            code: fields.next()?.to_string(),
            datatype: fields.next()?.to_string(),
            start: fields.nth(2)?.to_string(),
            end: fields.next()?.to_string(),
            size: fields.next()?.to_string(),
            depth: tabs,
        })
    }
}

impl Display for EurostatDataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}-{}-{}", String::from_utf8(vec![b' '; self.depth]).unwrap(), self.datatype, self.name, self.size)
    }
}

fn print_json(filename: &str, json: &str) {
    if let Ok(mut f) = File::create(filename) {
        let json: serde_json::Value = serde_json::from_str(json).unwrap();
        f.write_all(format!("{:#?}", json).as_bytes()).unwrap();

    }       
}

fn print_to_file(filename: &str, s: &impl Display) {
    if let Ok(mut f) = File::create(filename) {
        f.write_all(format!("{s}").as_bytes()).unwrap();
    }       
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let ui = MainWindow::new().unwrap();
    ui.window().set_size(PhysicalSize::new(1200, 900));
    let url = format!("{EUROSTAT_URL}catalogue/toc/txt");
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/plain"));
    let response = Client::new()
        .get(&url)
        .headers(headers.clone())
        .send()
        .await?;
    println!("{url}");
    
    let mut df = File::create("dataflow").unwrap();
    let mut pdf = File::create("parsed-dataflow").unwrap();

    for line in response.text().await?.lines() {
        df.write_all(format!("{}\n", line).as_bytes()).unwrap();
        if let Some(data) = EurostatDataset::try_from(line) {
            pdf.write_all(format!("{}\n", data).as_bytes()).unwrap();
        } else {
            println!("{line}");
        }
    }
    
    Ok(())
}

