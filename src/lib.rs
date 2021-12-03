use std::sync::{Arc, RwLock};

use json::JsonValue;
use json_parser::parse_js_to_json;
use neon::{prelude::*, result::Throw};
mod app;
mod error;
mod json_parser;
mod socket_instance;
mod util;

use socket_instance::{
    QuickSocketInstance, TcpChannelCreatePreferences, UdpChannelCreatePreferences,
};

use crate::json_parser::parse_json_to_js;

lazy_static::lazy_static! {
    static ref INSTANCE: Arc<RwLock<QuickSocketInstance>> = QuickSocketInstance::new();
}

pub static mut JS_HANDLER_CHANNEL: Option<neon::prelude::Channel> = None;
pub static mut JS_HANDLER_FUNCTION: Option<Root<JsFunction>> = None;

pub fn execute_js_handler(event: String, data: JsonValue) -> Result<(), String> {
    let channel: &neon::prelude::Channel = unsafe {
        match &JS_HANDLER_CHANNEL {
            Some(v) => v,
            None => panic!("Instance initialize invalid"),
        }
    };

    channel.send(move |mut cx| {
        let function = unsafe {
            match &JS_HANDLER_FUNCTION {
                Some(v) => v.to_inner(&mut cx),
                None => panic!("Instance initialize invalid"),
            }
        };

        let p_event: Handle<JsValue> = match cx.string(event).downcast(&mut cx) {
            Ok(v) => v,
            Err(_) => return cx.throw_error("error occured while downcast"),
        };

        let p_data = match match parse_json_to_js(&mut cx, data) {
            Ok(v) => v,
            Err(_) => return cx.throw_error("json parse fail"),
        }
        .downcast(&mut cx)
        {
            Ok(v) => v,
            Err(_) => return cx.throw_error("error occured while downcast"),
        };

        let undef_val = cx.undefined();
        if let Err(_) = function.call(&mut cx, undef_val, vec![p_event, p_data]) {
            // *error_ptr = Some(e.to_string())
        };

        Ok(())
    });
    Ok(())
}

fn create_tcp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let arg0 = cx.argument(0)?;
    let preferences = match TcpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    }; // Preferences

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };
    let channel = match write_locked.create_tcp_channel(|_| {}, preferences) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    drop(write_locked);

    let return_object = cx.empty_object();

    let port_value = cx.number(channel.port);
    return_object.set(&mut cx, "port", port_value)?;

    let uuid_value = cx.string(channel.channel_id.clone());
    return_object.set(&mut cx, "uuid", uuid_value)?;

    Ok(return_object)
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let arg0 = cx.argument(0)?;
    let preferences = match UdpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    }; // Preferences

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };
    let channel = match write_locked.create_udp_channel(|_| {}, preferences) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    drop(write_locked);

    let return_object = cx.empty_object();

    let port_value = cx.number(channel.port);
    return_object.set(&mut cx, "port", port_value)?;

    let uuid_value = cx.string(channel.channel_id.clone());
    return_object.set(&mut cx, "uuid", uuid_value)?;

    Ok(return_object)
}

fn event_handler_rs(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    //bridge
    let event: Handle<JsString> = cx.argument(0)?;
    let data: Handle<JsObject> = cx.argument(1)?;

    let parsed_event: String = event.value(&mut cx);
    let parsed_data: json::object::Object = parse_js_to_json(&mut cx, data)?;

    match app::bridge::resolver(parsed_event, parsed_data.into()) {
        Ok(_) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(e),
    }
}

fn set_js_event_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let func: Handle<JsFunction> = cx.argument(0)?;
    let func_origin = func.root(&mut cx);
    let channel = cx.channel();
    let undefined_value = cx.undefined();
    // let rs = cx.borrow_mut();
    unsafe { JS_HANDLER_CHANNEL = Some(channel) };
    unsafe { JS_HANDLER_FUNCTION = Some(func_origin) };

    Ok(undefined_value)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    cx.export_function("eventHandler", event_handler_rs)?;
    cx.export_function("setJsEventHandler", set_js_event_handler)?;

    Ok(())
}
