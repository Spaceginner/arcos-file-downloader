// todo custom-handle specific errors
use std::path::{Path, PathBuf};
use reqwest::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64ENGINE;
use crate::server::schemas::{DataResponse, FSTree, Session};

mod schemas;


#[derive(Debug)]
pub enum ServerCreationError {
    Connection,
    InvalidURL,
}


pub struct Server {
    client: reqwest::Client,
    base_url: url::Url,
}


impl Server {
    pub async fn new(mut base_url: url::Url, auth_code: Option<&str>) -> Result<Self, ServerCreationError> {
        if base_url.query().is_some() || base_url.cannot_be_a_base() {
            return Err(ServerCreationError::InvalidURL);
        };
        
        if let Some(auth_code) = auth_code {
            base_url.set_query(Some(&format!("ac={auth_code}")));
        };
        
        let self_ = Self {
            client: reqwest::Client::default(),
            base_url
        };
        
        if !self_.check_if_valid().await {
            return Err(ServerCreationError::Connection);
        };
        
        Ok(self_)
    }
    
    fn construct_url(&self, path: &str) -> url::Url {
        self.base_url.join(path).unwrap()
    }
    
    // xxx should it store the meta?
    pub async fn check_if_valid(&self) -> bool {
        let resp_res = self.client.execute(
            self.client.get(self.construct_url("/connect"))
                .build().unwrap()
        ).await;
        
        match resp_res {
            Err(_) => false,
            Ok(resp) => resp.status() == StatusCode::OK
        }
    }

    pub async fn auth_user<'a>(&'a self, username: &str, password: &str) -> Result<User<'a>, reqwest::Error> {
        let resp = self.client.execute(
            self.client.get(self.construct_url("/auth"))
                .basic_auth(username, Some(password))
                .build().unwrap()
        ).await.map_err(reqwest::Error::without_url)?;
        
        let token = resp.json::<DataResponse<Session>>().await?.data.token;
        
        Ok(User {
            server: self,
            username: username.to_string(),
            token
        })
    }
}


pub struct User<'a> {
    server: &'a Server,
    username: String,
    token: String,
}

impl<'a> User<'a> {
    pub fn fs(&'a self) -> FS<'a> {
        FS { user: self }
    }
    
    async fn auth_get(&self, path: &str, queries: Option<&[(&str, &str)]>) -> Result<reqwest::Response, reqwest::Error> {
        self.server.client.execute(
            self.server.client.get(self.server.construct_url(path))
                .bearer_auth(&self.token)
                .query(queries.unwrap_or(&[]))
                .build().unwrap()
        ).await.map_err(reqwest::Error::without_url)
    }
}


pub struct FS<'a> {
    user: &'a User<'a>
}


impl<'a> FS<'a> {
    pub async fn tree(&self) -> Result<Vec<PathBuf>, reqwest::Error> {
        let resp = self.user.auth_get("/fs/tree", None).await?;
        
        Ok(
            resp.json::<DataResponse<FSTree>>().await?.data.0.into_iter()
                .map(|pe| pe.scoped_path.into())
                .collect()
        )
    }
    
    pub async fn read(&self, path: &Path) -> Result<bytes::Bytes, reqwest::Error> {
        let resp = self.user.auth_get("/fs/file/get", Some(&[("path", &B64ENGINE.encode(path.to_str().unwrap()))])).await?;
        
        resp.bytes().await
    } 
}
