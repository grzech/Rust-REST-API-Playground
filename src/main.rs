use std::{collections::HashMap, fmt::Display, fs::File};

use reqwest::{self, Client, header::{ACCEPT, HeaderMap, HeaderValue}};
use serde::{Serialize, Deserialize};
use slint::{SharedString, PhysicalSize};
use std::io::prelude::*;

slint::include_modules!(); 

const UN_URL : &str = "https://data.un.org/ws/rest/";
const INDENT : &str = "    ";

#[derive(Deserialize, Serialize, Debug)]
struct DataFlowEntry {
    id: String,
    name: String,
    #[serde(rename = "agencyID")]
    agency_id: String,
    urn: String,
    version: String,
    structure: HashMap<String, String>,
}


#[derive(Deserialize, Serialize, Debug)]
struct CodeName {
    id: String,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CodeList {
    id: String,
    name: String,
    codes: Vec<CodeName>,
}

#[derive(Deserialize, Serialize, Debug)]
struct SchemeLists {
    //#[serde(rename = "conceptSchemes")]
    //concept_schemes: Vec<String>,
    codelists: Vec<CodeList>,
    //#[serde(rename = "dataStructures")]
    //data_structures: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct DataScheme {
    data: SchemeLists,
}

#[derive(Deserialize, Serialize, Debug)]
struct DataFlow {
    references: HashMap<String, DataFlowEntry>,
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

impl DataFlowEntry {
    pub fn get_structure_descriptor(&self) -> String {
        let descriptor = self.structure["urn"].split("=").last().unwrap();
        descriptor.replace(":", "/")
            .replace("(", "/")
            .replace(")", "")
    }

    pub fn get_data_descriptor(&self) -> String {
        let descriptor = self.urn.split("=").last().unwrap();
        descriptor.replace(":", ",")
            .replace("(", ",")
            .replace(")", "")
    }
}

impl Display for DataFlowEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{INDENT}{}{INDENT}{}{INDENT}\n{}{INDENT}\n{}", self.id, self.name, self.agency_id, self.urn, self.structure["urn"])
    }
}

impl Display for CodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lists = String::new();
        for list in self.codes.iter() {
            lists.push_str(&format!{"[{} {}]\n{INDENT}", list.id, list.name});
        }
        write!(f, "{} {}\n{INDENT}{lists}", self.id, self.name)
    }
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let ui = MainWindow::new().unwrap();
    ui.window().set_size(PhysicalSize::new(1200, 900));
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/json"));
    let response = Client::new()
        .get(format!("{UN_URL}dataflow"))
        .headers(headers.clone())
        .send()
        .await?;
    
    let response = response.text().await?;
    println!("{}", &response);
    let resp : DataFlow = serde_json::from_str(&response).unwrap();
    for (link, details) in resp.references {
        println!("{UN_URL}datastructure/{}?references=children", details.get_structure_descriptor());
        if details.get_structure_descriptor() == "WB/WDI/1.0" {
            let response = Client::new()
                .get(format!("{UN_URL}datastructure/{}?references=children", details.get_structure_descriptor()))
                .header(ACCEPT, "application/vnd.sdmx.structure+json;version=1.0.0-wd")
                .send()
                .await?;
            let json = response.text().await?;
            print_json("WB-WDI-1_0", &json);
            let json_struct: DataScheme = serde_json::from_str(&json).unwrap();
            let code_lists = json_struct.data.codelists;
            for list in code_lists {
                print_to_file(&list.id, &list);
            }
            println!("link: {}\nurn: {}", link, details.urn);
            let url = format!("{UN_URL}data/{}/A.POL.NY_GDP_MKTP_CD", details.get_data_descriptor());
            println!("{}", &url);
            let response = Client::new()
                .get(url)
                .header(ACCEPT, HeaderValue::from_static("text/json"))
                .send()
                .await?;
            let json = response.text().await?;
            print_json("data", &json);
        }
        
    }

    Ok(())
}