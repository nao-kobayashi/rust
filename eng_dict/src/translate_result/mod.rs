#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResult {
    pub result: String,
    pub tuc: Vec<Tuc>,
    pub phrase : Option<String>,
    pub from : Option<String>,
    pub dest : Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Tuc {
    pub phrase: Option<MeanItem>,
    pub meanings: Option<Vec<MeanItem>>,
    pub meaningId: Option<i64>,
    pub authors: Option<Vec<Option<i64>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeanItem {
    pub language: String,
    pub text: String,
}
/*
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFilter {
    pub phrase : String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub user : String,
    pub pass : String
}
*/