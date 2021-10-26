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
    
};

interface IUdpChannelCreatePreferences {

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
pub fn createTcpChannel(cfg: &ITcpChannelCreatePreferences) -> Result<(), JsValue> {
    let conf: TcpChannelCreatePreferences = match cfg.into_serde() {
        Ok(v) => v,
        Err(e) => return Err(JsValue::from_str("config is invalid")),
    };

    Ok(())
}

#[wasm_bindgen]
pub fn createUdpChannel(cfg: &IUdpChannelCreatePreferences) -> Result<(), JsValue> {
    let conf: UdpChannelCreatePreferences = match cfg.into_serde() {
        Ok(v) => v,
        Err(e) => return Err(JsValue::from_str("config is invalid")),
    };
    Ok(())
}

#[wasm_bindgen]
pub fn eventHandler(event: String, data: String) -> Result<(), JsValue> {
    Ok(())
}
