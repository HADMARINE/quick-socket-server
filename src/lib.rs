mod app;
mod error;
// mod js_interface;
mod socket_instance;
mod utils;

use std::sync::{Arc, RwLock};

use socket_instance::{
    QuickSocketInstance, TcpChannelCreatePreferences, UdpChannelCreatePreferences,
};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

lazy_static::lazy_static! {
    static ref INSTANCE: Arc<RwLock<QuickSocketInstance>> = QuickSocketInstance::new();
}

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACE_CHANNEL_CREATE_PREF: &'static str = r#"
interface ITcpChannelCreatePreferences {
    delete_client_when_closed: boolean;
    concurrent: boolean;
    preset?: string;
};

interface IUdpChannelCreatePreferences {
    delete_client_when_closed: boolean;
    preset?: string;
};

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ITcpChannelCreatePreferences")]
    pub type ITcpChannelCreatePreferences;
    #[wasm_bindgen(typescript_type = "IUdpChannelCreatePreferences")]
    pub type IUdpChannelCreatePreferences;
}

#[wasm_bindgen]
pub fn createTcpChannel(cfg: &ITcpChannelCreatePreferences) -> Result<u16, JsValue> {
    let conf: TcpChannelCreatePreferences = match cfg.into_serde() {
        Ok(v) => v,
        Err(e) => return Err(JsValue::from_str("config is invalid")),
    };

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(JsValue::from_str("failed to initialize channel")),
    };

    let channel = match write_locked.create_tcp_channel(|_| {}, conf) {
        Ok(v) => v,
        Err(_) => return Err(JsValue::from_str("failed to initialize channel")),
    };

    drop(write_locked);

    Ok(channel.port)
}

#[wasm_bindgen]
pub fn createUdpChannel(cfg: &IUdpChannelCreatePreferences) -> Result<u16, JsValue> {
    let conf: UdpChannelCreatePreferences = match cfg.into_serde() {
        Ok(v) => v,
        Err(e) => return Err(JsValue::from_str("config is invalid")),
    };

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(JsValue::from_str("failed to initialize channel")),
    };

    let channel = match write_locked.create_udp_channel(|_| {}, conf) {
        Ok(v) => v,
        Err(_) => return Err(JsValue::from_str("failed to initialize channel")),
    };

    drop(write_locked);

    Ok(channel.port)
}

#[wasm_bindgen]
pub fn eventHandler_rs(event: String, data: String) -> Result<(), JsValue> {
    app::bridge::resolver(event, data)
}

#[wasm_bindgen(module = "/resolver.js")]
extern "C" {
    pub fn resolver(event: String, data: String);
}
