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
}

impl EurostatDataset {
    fn try_from(s: &str) -> Option<Self> {
        let mut fields = s.split("\"\t\"").map(|x| x.replace("\"", "").trim().to_string());
        println!("{s}");
        println!("{:?}", fields);
        Some(EurostatDataset {
            name: fields.nth(0).unwrap().to_string(),
            code: fields.nth(1).unwrap().to_string(),
            datatype: fields.nth(2).unwrap().to_string(),
            start: fields.nth(5).unwrap().to_string(),
            end: fields.nth(6).unwrap().to_string(),
            size: fields.nth(7).unwrap().to_string(),
        })
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

    for line in response.text().await?.lines() {
        df.write_all(format!("{}\n", line).as_bytes()).unwrap();
        /*if let Some(data) = EurostatDataset::try_from(line) {
            df.write_all(format!("{}\t{}\t{}\n", data.name, data.code, data.datatype).as_bytes()).unwrap();
        }*/
    }
    
    Ok(())
}

