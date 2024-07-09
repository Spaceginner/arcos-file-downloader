use serde::Deserialize;


#[derive(Deserialize)]
pub struct DataResponse<T> {
    pub data: T,
}


#[derive(Deserialize)]
pub struct Session {
    pub username: String,
    pub token: String,
}


#[derive(Deserialize)]
pub struct FSPartialEntry {
    #[serde(rename = "scopedPath")]
    pub scoped_path: String,
    pub mime: String,
    pub filename: String,
}


#[derive(Deserialize)]
pub struct FSTree(pub Vec<FSPartialEntry>);
