extern crate iron;
extern crate router;
extern crate handlebars_iron;
extern crate params;
extern crate serde_urlencoded;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate bson;
extern crate mongodb;


use std::collections::HashMap;
use iron::prelude::*;
use iron::status;
use iron::Error;
use router::{Router, url_for};
use handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use mongodb::{ Client, ThreadedClient };
use mongodb::db::ThreadedDatabase;

#[derive(Debug, Serialize, Deserialize)]
struct TranslateResult {
    result: String,
    tuc: Vec<Tuc>,
    phrase : Option<String>,
    from : Option<String>,
    dest : Option<String>,
}

#[allow(non_snake_case)]
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
        let mut resp = Response::new();
        let mut data = HashMap::new();
        let url = format!("{}", url_for(req, "answer", HashMap::new()));
        data.insert(String::from("translate_path"), url);
        //data.insert(String::from("list"), );
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
        Ok(resp)
    }

    fn translate_handler(req: &mut Request) -> IronResult<Response> {
        println!("translate_path");
        use params::{Params, Value};
        let mut resp = Response::new();
        let mut data = HashMap::new();
        let url_ans = format!("{}", url_for(req, "answer", HashMap::new()));
        let url_index = format!("{}", url_for(req, "index", HashMap::new()));
        let map = &req.get_ref::<Params>().unwrap();

        let message = match map.find(&["word"]) {
            Some(&Value::String(ref word)) => { 
                get_translate_result(word.to_string())
            },
            _ => "".to_string()
        };

        data.insert(String::from("translate_path"), url_ans);
        data.insert(String::from("list_path"), url_index);
        data.insert(String::from("message"), format!("[{}]", message));
        resp.set_mut(Template::new("answer", data)).set_mut(status::Ok);

        Ok(resp)
    }

    /*fn get_translated_list() -> String {
        let client = Client::connect("192.168.56.2", 27017).expect("'failed to connect mongodb'");
        let coll = client.db("translate").collection("words_en");


        
    }*/

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
                    Err(e) => return format!("'get error(to text) {}'", e.to_string())
                }
            },
            Err(e) => return format!("'get error {}'", e.to_string())
        };


        let data: TranslateResult = match serde_json::from_str(json_str.as_str()) {
            Ok(j) => j,
            Err(e) => return format!("'serialize error {}'", e.to_string())
        };
        
        //save to mongodb
        match save_mongodb(&data) {
            Ok(()) => {},
            Err(e) => return e.to_string()
        }

        let mut result = "".to_string();
        for i in 0..data.tuc.len() {
            match &data.tuc[i].phrase {
                &Some(ref p) => {
                    if result != "" {
                        result = result + ",";
                    }
                    result = result + "'" + &p.text + "'";
                },
                &None => {}
            }
        }

        result
    }

    fn save_mongodb(json: &TranslateResult) -> Result<(), String> {
        let client = Client::connect("192.168.56.2", 27017).expect("'failed to connect mongodb'");
        let coll = client.db("translate").collection("words_en");

        match bson::to_bson(&json) {
            Ok(document) => {
                match document {
                    bson::Bson::Document(document_doc) => {
                        match coll.insert_one(document_doc, None) {
                            Ok(_) => {},
                            Err(e) => return Err(format!("'error mongodb insert.{}'", &e.to_string()))
                        }
                    },
                    _ =>  {
                        return Err(format!("'failed to create new document model.'"))
                    }
                }
            },
            Err(e) => {
                return Err(format!("'failed to create new document model.{}'", &e.to_string()));
            }
        }        

        Ok(())
    }

    //Create Router
    let mut router = Router::new();
    router.get("/", top_handler, "index");
    router.post("/answer", translate_handler, "answer");
    
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