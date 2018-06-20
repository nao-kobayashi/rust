use bson;
use translate_result::TranslateResult;
use mongodb::{ Client, ThreadedClient };
use mongodb::db::ThreadedDatabase;
use mongodb::coll::Collection;
use mongodb::cursor::Cursor;
use mongodb::coll::options::FindOptions;
use serde_json;

const MONGODB: &str = "192.168.56.2";
const LIST_LIMIT_DISP_WORDS: i32 = 15;

pub struct Mongo {
    coll: Collection,
}

impl Mongo {

    pub fn new() -> Mongo { 
        let client = Client::connect(MONGODB, 27017).expect("'failed to connect mongodb'");
        let collection = client.db("translate").collection("words_en");

        Mongo {
            coll: collection
        }
    }

    pub fn save_json(&self, json: &TranslateResult) -> Result<(), String> {
        let coll = &self.coll;
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

    pub fn check_exists(&self, phrase: String) -> Result<i64, String> {
        let coll = &self.coll;
        
        let bson_filter = self.get_bson_paramter("phrase".to_string(), phrase);
        let count = match coll.count(Some(bson_filter), None) {
            Ok(cursor) => cursor,
            Err(e) => return Err(format!("'error mongodb count.{}'", &e.to_string()))
        };

        Ok(count)
    }

    pub fn get_json(&self, phrase: String) -> Result<TranslateResult, String> {
        let cursor = match self.get_find_cursor(phrase) {
            Ok(cursor) => cursor,
            Err(e) => return Err(e),
        };

        for result in cursor { 
            if let Ok(item) = result {
                let bson_obj = bson::Bson::from(item);
                let json_obj: serde_json::value::Value = bson_obj.clone().into();
                match serde_json::from_value(json_obj) {
                    Ok(data) => return Ok(data),
                    Err(e) => return Err(format!("'error convert serde_json to object model. {}'", e.to_string()))
                }
            }
        }

        Err("No Data Found".to_string())
    }

    pub fn get_raw_json(&self, phrase: String) -> Result<serde_json::value::Value, String> {
        let cursor = match self.get_find_cursor(phrase) {
            Ok(cursor) => cursor,
            Err(e) => return Err(e),
        };

        for result in cursor { 
            if let Ok(item) = result {
                let bson_obj = bson::Bson::from(item);
                let json_obj: serde_json::value::Value = bson_obj.clone().into();
                match serde_json::from_value(json_obj) {
                    Ok(data) => return Ok(data),
                    Err(e) => return Err(format!("error convert serde_json to object model. {}", e.to_string()))
                }
            }
        }

        Err("No Data Found".to_string())
    }


    pub fn get_translated_list(&self) -> String {
        let coll = &self.coll;

        let mut param_doc = bson::ordered::OrderedDocument::new();
        param_doc.insert("phrase".to_string(), 1 as i32);

        let mut find_option = FindOptions::new();
        find_option.sort = Some(param_doc);

        let cursor = match coll.find(None, Some(find_option)) {
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
                    result_str = result_str + &convert_jsonlist_to_string(&data, LIST_LIMIT_DISP_WORDS);
                    result_str = result_str + &"]}".to_string();
                }
            }
        }

        result_str
    }

    fn get_bson_paramter(&self, col_name: String, filter: String) -> bson::Document {
        let mut param_doc = bson::ordered::OrderedDocument::new();
        param_doc.insert(col_name, filter);
        param_doc
    }

    fn get_find_cursor(&self, phrase: String) -> Result<Cursor, String> {
        let coll = &self.coll;

        let bson_filter = self.get_bson_paramter("phrase".to_string(), phrase);
        let cursor = match coll.find(Some(bson_filter), None) {
            Ok(cursor) => cursor,
            Err(e) => return Err(format!("'error mongodb find.{}'", &e.to_string()))
        };

        Ok(cursor)
    }
}


pub fn convert_jsonlist_to_string(data: &TranslateResult, limit: i32) -> String {
    let mut result = "".to_string();
    let mut count = 0;

    for i in 0..data.tuc.len() {
        if count >= limit { break; }

        match &data.tuc[i].phrase {
            &Some(ref p) => {
                if result != "" {
                    result = result + ",";
                }
                result = result + "'" + &p.text + "'";
            },
            &None => {}
        }

        count = count + 1;
    }

    result
}