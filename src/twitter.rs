
use anyhow::{Error, Result};

use reqwest::{
    blocking::{Client},
    header::{HeaderMap, HeaderName, HeaderValue},

};
use serde_json::{json, Value};
use std::{fs, io::prelude::*, path::Path};
use std::{str::FromStr, sync::Arc};
use std::collections::HashMap;


use reqwest_cookie_store::CookieStoreMutex;

const WEIBO_PIC_API: &str = "https://www.weibo.com/ajax/feed/groupstimeline?list_id=4070721837943483&refresh=4&fast_refresh=1&count=25";
const PAGES: i32 = 20;
fn main() -> Result<()> {
    // test();
    let max_id: Option<String> = Option::None;
    let n = 0;
    let max_id1 = download_one_page(max_id, n)?;
    println!("max_id:{:?}", max_id1);
    Ok(())
}
fn test() -> Result<()>{

    let client = Client::new();
    let resp = client. get(WEIBO_PIC_API)
        // .headers(get_headers().unwrap())
        .send()?;

    println!("{}", resp.status());
    println!("{:?}", &resp.json()?);
    // println!("{:?}", String::from_utf8(resp.bytes()?.to_vec()));
    Ok(())

}
fn download_one_page(max_id: Option<String>, mut n: i32) -> Result<Option<String>> {
    let client1 = get_client(get_headers()?)?;
    let mut url = String::new() + WEIBO_PIC_API;
    if max_id.is_some() {
        url = String::new() + WEIBO_PIC_API + "&max_id=" + &max_id.unwrap();
    }
    println!("n={},url=========={}", n, url);
    let mut response = client1.get(url)
        .headers(get_headers().unwrap())
        .send()?;
    let mut result:Value = Value::default();

    if response.status().is_success() {
        result = response.json()?;
    } else {
        let body = String::from_utf8(response.bytes().unwrap().to_vec())?;
        println!("{}", body);
        return Ok(None);
    }


    let mut list = Vec::new();
    if let Value::Array(posts) = result["statuses"].take() {
        for mut p in posts.into_iter() {
            // let pic_infos = p["pic_infos"].take();
            if let Value::Object(pic_infos) = p["pic_infos"].take() {
                if let Value::Array(pic_ids) = p["pic_ids"].take() {
                    for pi_id in pic_ids.into_iter() {
                        let id = match pi_id {
                            Value::String(id) => id,
                            _ => "".to_string(),
                        };
                        let pic = pic_infos.get(&id).unwrap();
                        let img_url = &pic["largest"]["url"];
                        if let Value::String(aa) = img_url {
                            list.push((aa.to_owned(), id));
                        }
                    }
                }
            }
        }
    }
    for (_i, (url, id)) in list.iter().enumerate() {
        let resp = client1.get(url).send()?;
        if resp.status().is_success() {
            let base_dir = String::new();
            let img_path = base_dir + "d:/Pictures/weiback/" + &id + ".jpg";
            let path = Path::new(&img_path);
            if !path.exists() {
                let mut pic_file: std::fs::File = std::fs::File::create(path)?;
                let _ = pic_file.write_all(resp.bytes()?.as_ref());
                println!("download:{}", &url);
            } else {
                println!("exist:{}", &url);
            }
        }
    }

    let max_id_option = &result["max_id_str"];
    if max_id_option.is_string() {
        let x = Some(max_id_option.as_str().unwrap().to_owned());
        n += 1;
        if n < PAGES {
            let y = download_one_page(x, n)?;
            return Ok(y);
        } else {
            return Ok(Option::None);
        }
    }
    Ok(Option::None)
}

pub fn get_headers() -> Result<HeaderMap> {
    let mut  headers = HeaderMap::new();
    let cookie = fs::read("cookie.json");
    if let Ok(ck) = cookie {
        let cookie_str :String = String::from_utf8(ck).unwrap();
        let header_map :HashMap<String,String> = serde_json::from_str(&cookie_str)?;
        for (k,v) in header_map {
            headers.insert(HeaderName::from_str(&k)?, HeaderValue::from_str(&v)?);
        }
    }

    Ok(headers)
}

pub fn get_client(header_map: HeaderMap) -> Result<Client> {

    let cookie_store = Arc::new(CookieStoreMutex::default());
    let client1 = Client::new();
    // let client = Client::builder()
    //     .cookie_store(true)
    //     // .cookie_provider(cookie_store.clone())
    //     .default_headers(header_map)
    //     
    //     .build()
    //     .unwrap();
    Ok(client1)
}
