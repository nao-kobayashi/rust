extern crate iron;
extern crate router;
extern crate handlebars_iron;
extern crate params;
extern crate serde_urlencoded;
extern crate reqwest;
extern crate eng_dict;
extern crate serde_json;
extern crate mount;
extern crate staticfile;

use std::collections::HashMap;
use iron::prelude::*;
use iron::status;
use iron::Error;
use iron::headers::Cookie;
use iron::headers::SetCookie;
use router::{Router, url_for};
use handlebars_iron::{Template, HandlebarsEngine, DirectorySource};
use eng_dict::translate_result::TranslateResult;
use eng_dict::mongo_db::{ Mongo, convert_jsonlist_to_string };
use mount::Mount;
use staticfile::Static;
use std::path::Path;

const AUTH_STR: &str = "test";

fn main() {

    fn top_handler(req: &mut Request) -> IronResult<Response> {
        match check_authorized(req) {
            None => return authrized_handler(req),
            _ => {},
        }        

        let mut resp = Response::new();
        let mut data = HashMap::new();
        let mongo = Mongo::new();

        let url = format!("{}", url_for(req, "answer", HashMap::new()));
        let url_detail = format!("'{}'", format!("{}", url_for(req, "detail", HashMap::new())) + "?word=");
        let word_list = format!("[{}]",  mongo.get_translated_list());

        data.insert(String::from("translate_path"), url);
        data.insert(String::from("detail_path"), url_detail);
        data.insert(String::from("list"), word_list);
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);

        Ok(resp)
    }

    fn translate_handler(req: &mut Request) -> IronResult<Response> {
        match check_authorized(req) {
            None => return authrized_handler(req),
            _ => {},
        }

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

    fn detail_handler(req: &mut Request) -> IronResult<Response> {
        match check_authorized(req) {
            None => return authrized_handler(req),
            _ => {},
        }

        use params::{Params, Value};
        let mut resp = Response::new();
        let mut data = HashMap::new();

        let url_ans = format!("{}", url_for(req, "answer", HashMap::new()));
        let url_index = format!("{}", url_for(req, "index", HashMap::new()));
        let map = &req.get_ref::<Params>().unwrap();

        let word = match map.find(&["word"]) {
            Some(&Value::String(ref word)) => { 
                word.to_string()
            },
            _ => "".to_string()
        };

        let json_obj = match get_translate_all(word){
            Ok(data) => data,
            Err(e) => 
            {
                println!("error at get_translate_all. {}", e);
                return Ok(resp);
            }
        };
        let json_str = json_obj.to_string().replace("\"", "'");

        data.insert(String::from("translate_path"), url_ans);
        data.insert(String::from("list_path"), url_index);
        data.insert(String::from("message"), json_str);
        resp.set_mut(Template::new("detail", data)).set_mut(status::Ok);

        Ok(resp)
    }

    fn check_authorized(req: &mut Request) -> Option<()> {
        match req.headers.get() {
            Some(&Cookie(ref cookie)) => {
                let mut c_cokkie = cookie.clone();
                match c_cokkie.pop() {
                    Some(str_cookie) => {
                        //println!("cookie:{:?}", str_cookie);
                        if str_cookie.contains(AUTH_STR) {
                            return Some(())
                        } else {
                            return None
                        }
                    },
                    _ => return None
                };
            },
            _ => return None
        }
    }

    fn authrized_handler(req: &mut Request) -> IronResult<Response> {
        use params::{Params, Value};
        let mut resp = Response::new();
        let mut data = HashMap::new();

        let url = format!("{}", url_for(req, "auth", HashMap::new()));
        let url_answer = format!("{}", url_for(req, "answer", HashMap::new()));
        let url_detail = format!("'{}'", format!("{}", url_for(req, "detail", HashMap::new())) + "?word=");
        let map = &req.get_ref::<Params>().unwrap();

        let user = match map.find(&["user"]) {
            Some(&Value::String(ref user)) => { 
                user.to_string()
            },
            _ => "".to_string()
        };

        let pass = match map.find(&["pass"]) {
            Some(&Value::String(ref pass)) => { 
                pass.to_string()
            },
            _ => "".to_string()
        };

        let mongo = Mongo::new();
        let count = match mongo.check_account(user.to_lowercase(), pass.to_lowercase()) {
            Some(count) => count,
            None => 0
        };

        println!("{} {} {}", user, pass, count);

        if count > 0 {
            let cookie = SetCookie(vec![String::from(AUTH_STR.to_string())]);
            resp.headers.set(cookie);

            let word_list = format!("[{}]",  mongo.get_translated_list());
            data.insert(String::from("translate_path"), url_answer);
            data.insert(String::from("detail_path"), url_detail);
            data.insert(String::from("list"), word_list);
            resp.set_mut(Template::new("index", data)).set_mut(status::Ok);

            Ok(resp)

        } else {

            data.insert(String::from("authrized_path"), url);
            resp.set_mut(Template::new("auth", data)).set_mut(status::Ok);
            Ok(resp)

        }
    }

    fn get_translate_all(word: String) -> Result<String, String> {
        let mongo = Mongo::new();

        match mongo.get_raw_json(word) {
            Ok(data) => {
                Ok(serde_json::to_string(&data).unwrap())
            },
            Err(e) => { 
                println!("parse error.{}", e);
                Err(e)
            },
        }
    }

    fn get_translate_result(word: String) -> String {
        let mongo = Mongo::new();
        //既に登録してる場合
        if let Ok(count) = mongo.check_exists(word.clone()) {
            if count > 0 {
                if let Ok(data) = mongo.get_json(word.clone()){
                    println!("read from mongodb phrase:{}", word.clone());
                    return convert_jsonlist_to_string(&data, 999);
                }
            }
        }

        //登録が無いため WEB APIで取得する。
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

        //結果の整形        
        if let Some(phrase) = data.phrase.clone() {
            if data.tuc.len() > 0  {
                match mongo.check_exists(phrase) {
                    Ok(count) => {
                        println!("record count:{}", count);
                        if count == 0 {
                            //save to mongodb
                            match mongo.save_json(&data) {
                                Ok(()) => {},
                                Err(e) => return e.to_string()
                            }
                        }
                    },
                    Err(e) => return e
                }
            }
        }

        convert_jsonlist_to_string(&data, 999)
    }


    //Create Router
    let mut router = Router::new();
    router.get("/", top_handler, "index");
    router.post("/answer", translate_handler, "answer");
    router.get("/detail", detail_handler, "detail");
    router.post("/auth", authrized_handler, "auth");
    
    //crate mount
    let mut mount = Mount::new();
    mount.mount("/", router)
        .mount("/css/", Static::new(Path::new("./src/css")));

    //Create Chain
    //let mut chain = Chain::new(router);
    let mut chain = Chain::new(mount);

    // Add HandlerbarsEngine to middleware Chain
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./src/templates/", ".hbs")));
    if let Err(r) = hbse.reload() {
        panic!("{}", r.description());
    }
    chain.link_after(hbse);
    
    println!("Listen on localhost:3000");
    let iron_instance = Iron::new(chain);
    println!("worker threads:{}", iron_instance.threads);
    iron_instance.http("localhost:3000").unwrap();
}