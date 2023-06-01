use std::{cell::RefCell, rc::Rc};

use js_sys::Date;
use log::{error, trace};
use sha3::{Digest, Sha3_256};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Element, HtmlElement, Request, RequestInit, RequestMode, Response, Window};

use crate::utils;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "localStorage"],js_name = getItem)]
    fn storage_get_item(key: &str) -> Option<String>;

    #[wasm_bindgen(js_namespace = ["window", "localStorage"],js_name = setItem)]
    fn storage_set_item(key: &str, value: &str);

    #[wasm_bindgen(js_namespace = ["window", "localStorage"],js_name = removeItem)]
    fn storage_remove_item(key: &str);
}

thread_local! {
    static LOCAL_VERSION: RefCell<Option<String>> = RefCell::new(None);
}

///
/// check update every ${frequency} milliseconds
/// default is 30 seconds
///
#[wasm_bindgen]
pub fn check_update(frequency: Option<i32>) {
    match setup_update(frequency.unwrap_or(30_000)) {
        Ok(_) => trace!("setup update success"),
        Err(e) => error!("setup update failed: {:?}", e),
    }
}

fn setup_update(frequency: i32) -> Result<(), JsValue> {
    let window = web_sys::window().expect("should have a window in this context");
    let document = window.document().expect("window should have a document");
    if let Some(updater) = document.get_element_by_id("fw_updater") {
        let origin_updater_btn = Rc::new(updater);
        let updater_btn = Rc::clone(&origin_updater_btn);
        let updater_fn = Closure::wrap(Box::new(move || {
            check_version_and_update(&updater_btn);
        }) as Box<dyn FnMut()>);
        check_version_and_update(&origin_updater_btn);
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            updater_fn.as_ref().unchecked_ref(),
            frequency,
        )?;
        updater_fn.forget();
    }
    Ok(())
}

fn check_version_and_update(updater_btn: &Rc<Element>) {
    let updater_btn = updater_btn.clone();
    spawn_local(async move {
        let mut local_version = LOCAL_VERSION.with(|v| v.borrow_mut().take());
        let origin_version = get_origin_version().await;
        trace!(
            "local_version is : {:?} origin_version: {:?}",
            local_version,
            origin_version
        );
        if origin_version.is_ok() {
            let origin_version = Some(origin_version.unwrap());
            if origin_version != local_version && local_version != None {
                updater_btn
                    .dyn_ref::<HtmlElement>()
                    .expect("#fw_updater be an `HtmlElement`")
                    .click();
            }
            local_version = origin_version;
        }
        LOCAL_VERSION.with(|v| *v.borrow_mut() = local_version.take());
    });
}

async fn get_origin_version() -> Result<String, JsValue> {
    let window = web_sys::window().expect("should have a window in this context");
    let url = get_version_url(&window);
    let code = match url {
        Ok(url) => {
            let version_code = get_version_code(url, window).await?;
            Ok(version_code)
        }
        Err(e) => Err(e),
    }?;

    match code.as_string() {
        Some(c) => {
            trace!("version data : {}", &c);
            if utils::is_number(&c) {
                Ok(sha_vers_code(&c))
            } else {
                Err(JsValue::from_str("version is invalid"))
            }
        }
        None => Err(JsValue::from_str("version query failed")),
    }
}

fn get_version_url(window: &Window) -> Result<String, JsValue> {
    let location = js_sys::Reflect::get(window, &JsValue::from_str("location"))?;
    let host = js_sys::Reflect::get(&location, &JsValue::from_str("host"))?
        .as_string()
        .expect("host should exist");

    let protocol = js_sys::Reflect::get(&location, &JsValue::from_str("protocol"))?
        .as_string()
        .unwrap_or(String::from("http"));
    let now = Date::new_0().value_of().to_string();
    Ok(format!(
        "{}//{}{}?_s={}",
        protocol, host, "/version.text", &now
    ))
}

async fn get_version_code(url: String, window: Window) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;
    if resp.status() == 200 {
        let text = JsFuture::from(resp.text()?).await?;
        return Ok(text);
    }
    let err_msg = format!("{} {}", resp.status(), resp.status_text());
    Err(JsValue::from_str(&err_msg))
}

fn sha_vers_code(version: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(version.as_bytes());
    let result = hasher.finalize();
    let a = format!("{:x}", result);
    a.to_uppercase()
}
