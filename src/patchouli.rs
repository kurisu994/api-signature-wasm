use std::sync::atomic::{AtomicI64, Ordering};

use js_sys::Date;
use log::{error, trace};
use md5::{Digest, Md5};
use rsa::rand_core::OsRng;
use rsa::{
    pkcs8::DecodePrivateKey, pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};

const PUB_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDhE47KJiREN8OCcOhYCCJMKusJ
VHHh8xl0IRaz86nPwnj+Qm1dssLn3Qr5x2Gn2oBCDXjMxQZ1WimEiNLSLJthZNr5
Qwqp1eZBwA4pYE0c8fQwm8Af+WB34XRwcGGYw9Kez2MWmSHNLIDp1MOdFx5iKAvS
TlFrlr7/CdJfo5qpNwIDAQAB
-----END PUBLIC KEY-----";

const PRI_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBAOETjsomJEQ3w4Jw
6FgIIkwq6wlUceHzGXQhFrPzqc/CeP5CbV2ywufdCvnHYafagEINeMzFBnVaKYSI
0tIsm2Fk2vlDCqnV5kHADilgTRzx9DCbwB/5YHfhdHBwYZjD0p7PYxaZIc0sgOnU
w50XHmIoC9JOUWuWvv8J0l+jmqk3AgMBAAECgYEAhX8LJx0eT5Pfk0OSm3wfk5u1
EjCS51g/1aA2jf9Mzdxj1TLjGTsI3Ws+sk2uv1ca0ZKZ/qIxpXMXwvNAT8aDVVEz
+zSfIXE/U++pMkd8KTxtWiKinm92jn9y4JPXiFl0kWE7viLr1wapskXIp6Vhh70k
0tgqd3BdJ/xMTW+igZkCQQDwXdS1Y1Wzlwxg2ykxCH4ypTnJmFMNirk6Q7zYNIPa
YKJApKjRr4o7zVucxkF0swAiaOHRuurCE2wkecON7hMLAkEA77ckWg69ZM1SBvKN
N4fKmU3T4gQSUqCkmxpIc1PMgbmEVxKTAuW0segjP2EoW0OusMKtRxvDwfSucr/z
WsseBQJBAMffPmEWmM0dbU2c8EO1rDqw6byYzXcVQ7EPYpjmEj4k3MakJT03mtrf
iK50rTk9H399d0nPBCcdv28VUWdT8MECQQCCBUk/W71YnpE+WXNFSm8WhgNGFUVG
8gE2a6QegbZsKo7gl5+Ls8I1uR7dMrqr/eMT1xQbfLDKVAgHD5xUg9VtAkAjgClX
L3woK+Knq3RBQdX2SX8ptxtE43QYklaA/AV42kwtwi42tA7x3hcTtzQ9vDkk6W+k
wLZeoI4zQonxrj6r
-----END PRIVATE KEY-----";

static SERVER_TIME: AtomicI64 = AtomicI64::new(0);
static LOCAL_TIME: AtomicI64 = AtomicI64::new(0);

#[derive(Serialize, Deserialize, Debug)]
pub struct Unify {
    pub p: String,
    pub t: i64,
    pub u: String,
}

#[wasm_bindgen]
pub async fn encrypt(path: Option<String>) -> Result<String, JsValue> {
    trace!("path is {:?}", path);
    if let Some(path) = path {
        let window = web_sys::window();
        let sign = match window {
            Some(win) => {
                check_server_time(&win).await;
                encrypt_api(&path, &win)
            }
            None => Err(JsValue::from_str("window not found")),
        };
        if let Ok(sign) = sign {
            return Ok(sign);
        }
        error!("encrypt api failed: {:?}", sign)
    }
    Ok(String::from(""))
}

#[wasm_bindgen]
pub fn decrypt(sign: Option<String>) -> Result<String, JsValue> {
    trace!("sign is {:?}", sign);
    if let Some(_sign) = sign {
        let sign_vec = crate::utils::base64_decode(&_sign);
        if sign_vec.is_none() {
            return Ok(String::from(""));
        }
        let prv_key = RsaPrivateKey::from_pkcs8_pem(PRI_PEM).unwrap();
        let origin_data = prv_key
            .decrypt(Pkcs1v15Encrypt, &sign_vec.unwrap())
            .unwrap();
        let origin_str: String = String::from_utf8_lossy(&origin_data).to_string();
        println!("{:?}", origin_str);
        return Ok(origin_str);
    }
    Ok(String::from(""))
}

#[wasm_bindgen]
pub async fn async_time() -> Result<(), JsValue> {
    match web_sys::window() {
        Some(win) => {
            check_server_time(&win).await;
            Ok(())
        }
        None => {
            error!("{}", "window not found");
            Err(JsValue::from_str("window not found"))
        }
    }
}

pub async fn check_server_time(win: &Window) {
    if SERVER_TIME.load(Ordering::SeqCst) > 0 {
        return;
    }
    if let Ok(url) = get_url() {
        let data = get_server_time(&url, win).await;
        if let Ok(data) = data {
            SERVER_TIME.swap(data, Ordering::SeqCst);
            LOCAL_TIME.swap(Date::now() as i64, Ordering::SeqCst);
            return;
        }
        error!("{:?}", data);
    }
}

fn encrypt_api(unify_path: &str, win: &Window) -> Result<String, JsValue> {
    let ua = get_ua(&win)?;
    let ua_md5 = get_md5_middle16(&ua);
    let path_md5 = get_md5_middle16(unify_path);
    let now = get_now_time();
    let unify = Unify {
        p: path_md5,
        t: now,
        u: ua_md5,
    };
    let a = encrypt_unify(unify);
    Ok(a)
}

fn encrypt_unify(data: Unify) -> String {
    let unify_str = format!("{}#{}#{}", data.p, data.t, data.u);
    let pub_key = RsaPublicKey::from_public_key_pem(PUB_PEM).unwrap();
    let enc_data = pub_key
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, &unify_str.as_bytes())
        .expect("failed to encrypt");
    let sign = crate::utils::base64_encode(&enc_data);
    sign
}

fn get_md5_middle16(data: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(data.as_bytes());
    let result = format!("{:x}", hasher.finalize());
    let result = &result[8..24];
    result.to_string()
}

fn get_ua(window: &Window) -> Result<String, JsValue> {
    let navigator = js_sys::Reflect::get(window, &JsValue::from_str("navigator"))?;
    let ua = js_sys::Reflect::get(&navigator, &JsValue::from_str("userAgent"))?
        .as_string()
        .unwrap_or(String::from("no_user_agent"));
    Ok(ua)
}

fn get_now_time() -> i64 {
    let local_now = Date::now() as i64;
    let initial_local_time = LOCAL_TIME.load(Ordering::SeqCst);
    let initial_sever_time = SERVER_TIME.load(Ordering::SeqCst);
    trace!(
        "initial_local_time: {}, initial_sever_time: {}",
        initial_local_time,
        initial_sever_time
    );
    local_now - initial_local_time + initial_sever_time
}

fn get_url() -> Result<String, JsValue> {
    let now = Date::new_0().value_of().to_string();
    Ok(format!(
        "https://api.m.taobao.com/rest/api3.do?api={}&_s_s={}",
        "mtop.common.getTimestamp", &now
    ))
}

async fn get_server_time(url: &str, window: &Window) -> Result<i64, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::NoCors);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;
    if resp.status() == 200 {
        let data = JsFuture::from(resp.json()?).await?;
        let time_data = js_sys::Reflect::get(&data, &JsValue::from_str("data"))?;
        trace!("{:?}", time_data);
        let time = js_sys::Reflect::get(&time_data, &JsValue::from_str("t"))?;
        if let Some(time) = time.as_string() {
          let _time: i64 = time.parse().unwrap();
          return Ok(_time);
        }
    }
    let err_msg = format!("{} {}", resp.status(), resp.status_text());
    Err(JsValue::from_str(&err_msg))
}
