use anyhow::Result;
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use serde_json::{json, Value};
use std::{io::prelude::*, path::Path};
use std::{str::FromStr, sync::Arc};

use reqwest_cookie_store::CookieStoreMutex;

const WEIBO_PIC_API: &str = "https://www.weibo.com/ajax/feed/groupstimeline?list_id=4070721837943483&refresh=4&fast_refresh=1&count=25";
const PAGES: i32 = 20;
fn main() -> Result<()> {
    let max_id: Option<String> = Option::None;
    let mut n = 0;

    let max_id1 = downloadOnePage(max_id, n)?;

    println!("max_id:{:?}", max_id1);
    Ok(())
}
fn downloadOnePage(max_id: Option<String>, mut n: i32) -> Result<Option<String>> {
    let client1 = get_client(get_headers1()?)?;
    let img = "https://wx3.sinaimg.cn/large/a0d77288ly1hos9my87vsj20zj1hc7eg.jpg";
    let mut url = String::new() + WEIBO_PIC_API;
    if max_id.is_some() {
        url = String::new() + WEIBO_PIC_API + "&max_id=" + &max_id.unwrap();
    }
    println!("n={},url=========={}", n, url);
    let mut result: Value = client1.get(url).send().expect("http error").json()?;

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
    for (i, (url, id)) in list.iter().enumerate() {
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
            let y = downloadOnePage(x, n)?;
            return Ok(y);
        } else {
            return Ok(Option::None);
        }
    }
    Ok(Option::None)
}

pub fn get_headers1() -> Result<Value> {
    let headers = json!(
        {
            "accept": "application/json, text/plain, */*",
            "accept-language": "zh-CN,zh;q=0.9",
            "client-version": "v2.45.7",
            "priority": "u=1, i",
            "sec-ch-ua": "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"",
            "sec-ch-ua-mobile": "?0",
            "sec-ch-ua-platform": "\"Windows\"",
            "sec-fetch-dest": "empty",
            "sec-fetch-mode": "cors",
            "sec-fetch-site": "same-origin",
            "server-version": "v2024.04.29.1",
            "x-requested-with": "XMLHttpRequest",
            "x-xsrf-token": "GQKKO02vZOkV_yqvsBUGDWEM",
            "cookie": "login_sid_t=82cea204daa23e85152ed298dde251bc; cross_origin_proto=SSL; _s_tentry=weibo.com; Apache=6046515131505.311.1653830647774; SINAGLOBAL=6046515131505.311.1653830647774; XSRF-TOKEN=GQKKO02vZOkV_yqvsBUGDWEM; UOR=,,login.sina.com.cn; SSOLoginState=1682605537; ULV=1685243937594:1:1:1:6046515131505.311.1653830647774:; SCF=ArkeVXUTqa_IfHxK4k7cfmAC-jkG52QdK7QxXl4JP2J2zXyRR2Qv6qnLJUxxFha4iReWerzTdy5kHBBFTd-xfn4.; ALF=1716648455; SUB=_2A25LLh1XDeRhGedJ7lMS9CbEzjyIHXVoQhCfrDV8PUJbkNB-LU79kW1NUeGMq2L9-PQhkwZvCCuvu77ZzHM53lzz; SUBP=0033WrSXqPxfM725Ws9jqgMF55529P9D9WhY9rkcIGocADgl6y8XwzKX5JpX5KMhUgL.Fo2NSK20ShnRSK52dJLoI7_0UPWLMJyfeo5p15tt; WBPSESS=HRsQ-3pQNdFRfLXEGcltKV7a3vOMM5uiyYoeXuddqb9Z563hBfC_V-dzoPnivzi1qGEv17rJoaeKJw_YfcvnevX-NufGSPvSVx2wGSzXRAM23TOonE89-6eXflxRkVTOPninlZj3nBjBPJzTcf4r-Q==",
            "Referer": "https://www.weibo.com/mygroups?gid=4070721837943483",
            "Referrer-Policy": "strict-origin-when-cross-origin"
          }
    );
    Ok(headers)
}

pub fn get_client(mut headers: Value) -> Result<Client> {
    let mut list = Vec::new();
    if let Value::Object(map) = headers.take() {
        for (k, v) in map {
            let str_value = match v {
                Value::String(sv) => sv,
                _ => String::new(),
            };
            let a = (
                HeaderName::from_str(&k)?,
                HeaderValue::from_str(&str_value)?,
            );
            list.push(a);
        }
    }
    let header_map = HeaderMap::from_iter(list);

    let cookie_store = Arc::new(CookieStoreMutex::default());
    let client = Client::builder()
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .default_headers(header_map)
        .build()
        .unwrap();
    Ok(client)
}
