use std::path::PathBuf;


#[derive(gumdrop::Options)]
pub struct AppOptions {
    #[options(help = "print the help message")]
    pub help: bool,

    #[options(help = "api to use", default = "https://community.arcapi.nl")]
    pub api: url::Url,
    
    #[options(help = "auth code for the api")]
    pub code: Option<String>,
    
    #[options(required, help = "name of the account to download files from")]
    pub username: String,
    
    #[options(required, help = "the password to the specified account")]
    pub password: String,
    
    #[options(help = "the output directory to which files will be downloaded", default = "files")]
    pub out: PathBuf,
}
