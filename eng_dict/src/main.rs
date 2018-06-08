extern crate iron;
extern crate router;
extern crate handlebars_iron;
extern crate params;
extern crate serde_urlencoded;
extern crate reqwest;

use std::collections::HashMap;
use std::error::Error;
use iron::prelude::*;
use iron::status;
use router::{Router, url_for};
use handlebars_iron::{Template, HandlebarsEngine, DirectorySource};

fn main() {

    fn top_handler(req: &mut Request) -> IronResult<Response> {
        println!("top_handler");
        let url = format!("{}", url_for(req, "translate", HashMap::new()));
        return Ok(get_index_page(url, "".to_string()));
    }

    fn get_index_page(url: String, message: String) -> Response {
        let mut resp = Response::new();
        let mut data = HashMap::new();

        let result = if message != "" {
            get_translate_result(message)
        } else {
            message
        };

        data.insert(String::from("translate_path"), url);
        data.insert(String::from("message"), format!("'{}'", result));
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
        resp
    }

    fn translate_handler(req: &mut Request) -> IronResult<Response> {
        println!("translate_path");
        use params::{Params, Value};
        let url = format!("{}", url_for(req, "translate", HashMap::new()));
        let map = &req.get_ref::<Params>().unwrap();
        return match map.find(&["word"]) {
            Some(&Value::String(ref word)) => Ok(get_index_page(url, word.to_string())),
            _ => Ok(get_index_page(url, "".to_string()))
        }
    }

    fn get_translate_result(word: String) -> String {
        let enc_word = match serde_urlencoded::to_string(word.clone()) {
            Ok(word) => word,
            Err(_e) => word
        };

        let url = format!("https://glosbe.com/gapi/translate?from=eng&dest=jpn&format=json&phrase={}&pretty=false", enc_word);
        match reqwest::get(url.as_str()) {
            Ok(mut req) => {
                return match req.text() {
                    Ok(t) => t,
                    Err(e) => e.to_string()
                }
            },
            Err(e) => return e.to_string()
        }
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