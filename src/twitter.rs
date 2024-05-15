use std::{fs, io::prelude::*};
use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};


const TWITTER_URI: &str = "https://video.twimg.com/ext_tw_video/1399524962646462466/pu/vid/0/3000/720x1280/6HSBTv0GxP4nEv5L.ts";
fn main() -> Result<()> {
    let video_text = fs::read_to_string("video.txt").unwrap();
    let list:Vec<&str> = video_text.split("\n").collect();
    for uri in list {
        get(uri);
    }
    // get(TWITTER_URI);
    Ok(())
}
pub fn get(url: &str) {
    println!("download:{}", url);
    let array :Vec<&str> = url.rsplit("/").collect();
    let filename  = array.get(0).unwrap();
    let client = Client::new();
    let mut resp = client. get(url)
        // .headers(get_headers().unwrap())
        .send()
        .expect("11")
        .bytes()
        .expect("22")
        ;
    fs::write(filename, resp.as_ref()).expect("fail");
}


pub fn get_headers() -> Result<HeaderMap> {
    let mut  headers = HeaderMap::new();
    let cookie = fs::read("twitter_cookie.json");
    if let Ok(ck) = cookie {
        let cookie_str :String = String::from_utf8(ck).unwrap();
        let header_map :HashMap<String,String> = serde_json::from_str(&cookie_str)?;
        for (k,v) in header_map {
            headers.insert(HeaderName::from_str(&k)?, HeaderValue::from_str(&v)?);
        }
    }
    println!("{:?}", &headers);
    Ok(headers)
}


