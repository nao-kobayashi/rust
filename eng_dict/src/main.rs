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

#[derive(Debug, Serialize, Deserialize)]
struct SearchFilter {
    phrase : String
}

//todo
//既にデータがあったらWEB apiを投げたくない。
//構造体のところとかリファクタする。
fn main() {

    fn top_handler(req: &mut Request) -> IronResult<Response> {
        println!("top_handler");
        let mut resp = Response::new();
        let mut data = HashMap::new();
        let url = format!("{}", url_for(req, "answer", HashMap::new()));
        let word_list = format!("[{}]",  get_translated_list());
        data.insert(String::from("translate_path"), url);
        data.insert(String::from("list"), word_list);
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

    fn get_translated_list() -> String {
        let client = Client::connect("192.168.56.2", 27017).expect("'failed to connect mongodb'");
        let coll = client.db("translate").collection("words_en");

        let cursor = match coll.find(None, None) {
            Ok(cursor) => cursor,
            Err(e) => {
                return format!("error mongodb find. {:?}", e);
            }
        };

        let mut result_str = "".to_string();
        for result in cursor {
            if let Ok(item) = result {
                let bson_obj = bson::Bson::from(item);
                let json_obj: serde_json::value::Value = bson_obj.clone().into();
                let data: TranslateResult = match serde_json::from_value(json_obj) {
                    Ok(data) => data,
                    Err(e) => return format!("error convert serde_json to object model. {}", e.to_string())
                };
                
                if let Some(s) = data.phrase.clone() {
                    if result_str != "" {
                        result_str = result_str + ",";
                    }
                    result_str = result_str + &"{'phrase':'".to_string();
                    result_str = result_str + &s;
                    result_str = result_str + &"','words':[".to_string();
                    result_str = result_str + &translate_result(&data);
                    result_str = result_str + &"]}".to_string();
                }
            }
        }

        result_str
    }

    fn get_translate_result(word: String) -> String {
        let enc_word = match serde_urlencoded::to_string(word.clone()) {
            Ok(word) => word.trim().to_string(),
            Err(_e) => word.trim().to_string()
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
        
        if let Some(phrase) = data.phrase.clone() {
            if data.tuc.len() > 0  {
                match check_exists(phrase) {
                    Ok(count) => {
                        println!("record count:{}", count);
                        if count == 0 {
                            //save to mongodb
                            match save_mongodb(&data) {
                                Ok(()) => {},
                                Err(e) => return e.to_string()
                            }
                        }
                    },
                    Err(e) => return e
                }
            }
        }

        translate_result(&data)
    }

    fn translate_result(data: &TranslateResult) -> String {
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

    fn check_exists(phrase: String) -> Result<i64, String> {
        let client = Client::connect("192.168.56.2", 27017).expect("'failed to connect mongodb'");
        let coll = client.db("translate").collection("words_en");

        let filter_str = format!("{{\"phrase\":\"{}\"}}", phrase);
        println!("{}", filter_str);
        let filter: SearchFilter =  match serde_json::from_str(filter_str.as_str()) {
            Ok(j) => j,
            Err(e) => return Err(format!("'serialize error at check_exists. {}'", e.to_string()))
        };

        let count = match bson::to_bson(&filter) {
            Ok(filter_document) => {
                match filter_document {
                    bson::Bson::Document(filter_document_doc) => {
                        match coll.count(Some(filter_document_doc), None) {
                            Ok(count) => count,
                            Err(e) => return Err(format!("'error mongodb count.{}'", &e.to_string()))
                        }
                    },
                    _ =>  {
                        return Err(format!("'failed to create filter document model.'"))
                    }
                }
            },
            Err(e) => {
                return Err(format!("'failed to create filter document model.{}'", &e.to_string()));
            }

        };

        Ok(count)
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