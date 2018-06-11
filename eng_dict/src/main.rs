extern crate iron;
extern crate router;
extern crate handlebars_iron;
extern crate params;
extern crate serde_urlencoded;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use iron::prelude::*;
use iron::status;
use iron::Error;
use router::{Router, url_for};
use handlebars_iron::{Template, HandlebarsEngine, DirectorySource};

#[derive(Debug, Serialize, Deserialize)]
struct TranslateResult {
    result: String,
    tuc: Vec<Tuc>,
    phrase : Option<String>,
    from : Option<String>,
    dest : Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tuc {
    phrase: Option<MeanItem>,
    meanings: Option<Vec<MeanItem>>,
    meaningId: Option<i64>,
    authors: Option<Vec<Option<i64>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MeanItem {
    language: String,
    text: String,
}

fn main() {

    fn top_handler(req: &mut Request) -> IronResult<Response> {
        println!("top_handler");
        let url = format!("{}", url_for(req, "translate", HashMap::new()));
        return Ok(get_index_page(url, "".to_string()));
    }

    fn get_index_page(url: String, message: String) -> Response {
        let mut resp = Response::new();
        let mut data = HashMap::new();
        data.insert(String::from("translate_path"), url);
        data.insert(String::from("message"), format!("[{}]", message));
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
        resp
    }

    fn translate_handler(req: &mut Request) -> IronResult<Response> {
        println!("translate_path");
        use params::{Params, Value};
        let url = format!("{}", url_for(req, "translate", HashMap::new()));
        let map = &req.get_ref::<Params>().unwrap();

        return match map.find(&["word"]) {
            Some(&Value::String(ref word)) => { 
                Ok(get_index_page(url, get_translate_result(word.to_string())))
            },
            _ => Ok(get_index_page(url, "".to_string()))
        }
    }

    fn get_translate_result(word: String) -> String {
        let enc_word = match serde_urlencoded::to_string(word.clone()) {
            Ok(word) => word,
            Err(_e) => word
        };

        let url = format!("https://glosbe.com/gapi/translate?from=en&dest=ja&format=json&phrase={}&pretty=false", enc_word);
        let json_str = match reqwest::get(url.as_str()) {
            Ok(mut req) => {
                match req.text() {
                    Ok(t) => t,
                    Err(e) => return format!("get error(to text) {}", e.to_string())
                }
            },
            Err(e) => return format!("get error {}", e.to_string())
        };

        let data: TranslateResult = match serde_json::from_str(json_str.as_str()) {
            Ok(j) => j,
            Err(e) => return format!("serialize error {}", e.to_string())
        };

        let mut result = "".to_string();
        for i in 0..data.tuc.len() {
            match &data.tuc[i].phrase {
                Some(ref p) => {
                    if result != "" {
                        result = result + ",";
                    }
                    result = result + "'" + &p.text + "'";
                },
                None => {}
            }
        }

        result
    }

    //Create Router
    let mut router = Router::new();
    router.get("/", top_handler, "index");
    router.post("/answer", translate_handler, "translate");
    
    //Create Chain
    let mut chain = Chain::new(router);

    // Add HandlerbarsEngine to middleware Chain
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./src/templates/", ".hbs")));
    if let Err(r) = hbse.reload() {
        panic!("{}", r.description());
    }
    chain.link_after(hbse);
    
    println!("Listen on localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}