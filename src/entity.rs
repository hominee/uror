use crate::schema::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
#[cfg(feature = "obfs")]
use harsh::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::{Arc, RwLock},
};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
const DEFAULT_HASH_LEN: usize = 12;

#[derive(Deserialize, Serialize, Insertable, Queryable)]
#[table_name = "uris"]
pub struct Data {
    //#[serde(default = "Default::default", skip_serializing_if = "Option::is_none")]
    //id: Option<i32>,
    /// the original uri
    uri: String,
    /// the short uri
    iner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Uri {
    pub uri: String,
}

#[derive(Clone)]
pub struct Actor {
    cached: Arc<RwLock<HashMap<String, String>>>,
    keys: Arc<RwLock<VecDeque<String>>>,
    conn: Arc<SqliteConnection>,
    #[cfg(feature = "obfs")]
    inner: Arc<Harsh>,
}

impl Actor {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        #[cfg(feature = "obfs")]
        let salt = env::var("SALT").expect("SALT is not set");
        Self {
            cached: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(VecDeque::new())),
            conn: Arc::new(
                SqliteConnection::establish(&database_url)
                    .expect("Unable to connect to local database"),
            ),
            #[cfg(feature = "obfs")]
            inner: Arc::new(
                Harsh::builder()
                    .salt(salt)
                    .build()
                    .expect("Unable to initialize the Harsh"),
            ),
        }
    }

    /// use `seed` to generate a base64 hash
    fn hash_encode(&self, seed: &str) -> String {
        use std::{collections::hash_map::DefaultHasher, hash::Hasher};
        dotenv::dotenv().ok();
        let mut hasher = DefaultHasher::new();
        let hash_len = env::var("URI_LEN")
            .and_then(|en| Ok(en.parse::<usize>().unwrap()))
            .unwrap_or(DEFAULT_HASH_LEN);
        assert!(hash_len > 5, "uri length must greater than 5");
        let mut slice = Vec::with_capacity(hash_len);
        for _ in 0..hash_len {
            hasher.write(seed.as_bytes());
            let e = (hasher.finish() % 64) as u8;
            slice.push(e);
        }
        Self::byte2b64(&slice)
    }

    pub fn byte2b64(byte: &[u8]) -> String {
        let len = byte.len();
        let mut s = Vec::with_capacity(len);
        for i in 0..len {
            let e = byte[i];
            if e < 10 {
                s.push(e + 48);
            } else if e < 36 {
                s.push(e + 55);
            } else if e < 62 {
                s.push(e - 36 + 97);
            } else if e == 62 {
                s.push(45);
            } else {
                assert_eq!(e, 63, "must less than 64");
                s.push(95);
            }
        }
        std::string::String::from_utf8(s).unwrap()
    }

    pub fn encode(&mut self, s: &String) -> String {
        #[cfg(feature = "obfs")]
        {
            log::debug!("obfscate uri: {}", s);
            let bytes = s.as_bytes().iter().map(|e| *e as u64).collect::<Vec<u64>>();
            return self.inner.encode(&bytes);
        }
        #[cfg(feature = "default")]
        #[allow(unreachable_code)]
        self.hash_encode(s)
    }

    pub fn cache(&mut self, key: String, value: String) {
        dotenv::dotenv().ok();
        let buffer_len = env::var("BUFFER_LEN")
            .and_then(|en| Ok(en.parse::<usize>().unwrap()))
            .unwrap_or(1000);
        if self.keys.read().unwrap().len() > buffer_len {
            //Arc::make_mut(&mut self.keys).pop_front();
            self.keys.write().unwrap().pop_front();
        }
        log::debug!("cached key: {}, value: {}", &key, &value);
        //(*Arc::make_mut(&mut self.keys)).push_back(key.clone());
        //(*Arc::make_mut(&mut self.cached)).insert(key, value);
        self.keys.write().unwrap().push_back(key.clone());
        self.cached.write().unwrap().insert(key, value);
    }

    pub fn read(&mut self, uri_: &str) -> Result<String, GenericError> {
        use crate::schema::uris::dsl::*;
        let cached = self.cached.read().unwrap();
        let val = cached.get(uri_);
        let mut cache2 = false;
        if val.is_some() {
            log::debug!("load {} from cache", val.as_ref().unwrap());
            cache2 = true;
            return Ok(val.unwrap().into());
        }
        drop(cached);
        uris.filter(uri.eq(uri_))
            .get_result::<Data>(&*self.conn)
            .and_then(|en| {
                if !cache2 {
                    self.cache(uri_.to_owned(), en.iner.clone());
                }
                Ok(en.iner)
            })
            .map_err(|e| e.into())
    }

    pub fn insert(&mut self, iner_: String) -> Result<String, GenericError> {
        use crate::schema::uris::dsl::*;
        let uri_ = self.encode(&iner_);
        let cached = self.cached.read().unwrap();
        let val = cached.get(&uri_);
        let mut cache2 = false;
        if val.is_some() {
            log::debug!("load {} from cache", val.as_ref().unwrap());
            cache2 = true;
            return Ok(val.unwrap().into());
        }
        drop(cached);
        diesel::insert_or_ignore_into(uris)
            .values((uri.eq(uri_.clone()), iner.eq(iner_.clone())))
            .execute(&*self.conn)
            .and_then(|_| {
                if !cache2 {
                    self.cache(uri_.clone(), iner_);
                }
                Ok(uri_)
            })
            .map_err(|e| e.into())
    }

    pub fn delete(&mut self, uri_: String) -> Result<bool, GenericError> {
        use crate::schema::uris::dsl::*;
        diesel::delete(uris)
            .filter(uri.eq(uri_.clone()))
            .execute(&*self.conn)
            .and_then(|_| {
                self.cached.write().unwrap().remove(&uri_);
                self.keys.write().unwrap().retain(|e| e != &uri_);
                Ok(true)
            })
            .map_err(|e| e.into())
    }
}

unsafe impl Send for Actor {}
unsafe impl Sync for Actor {}

#[test]
fn test_hash_encode() {
    let s = "Hello World";
    let mut actor = Actor::new();
    let r = actor.hash_encode(s);
    dbg!(&r);
    //assert!(false);
}
