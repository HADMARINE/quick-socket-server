use json::JsonValue;
use std::collections::HashMap;

pub type BridgeMapType = HashMap<String, BridgeHandlerType>;

pub type BridgeHandlerType = Box<dyn Fn(JsonValue) -> Result<(), Box<dyn std::error::Error>>>;

pub fn manager() -> BridgeMapType {
    // return match preset.as_str() {
    //     "none" => HashMap::new(),
    //     "echo" => echo::get(),
    //     _ => {
    //         panic!("Invalid preset : {}", preset);
    //     }
    // };
    let mut map: BridgeMapType = HashMap::new(); // TODO : Complete this

    map.insert(String::from("print"), Box::new(print));
    map
}

pub fn resolver(event: String, data: JsonValue) -> Result<(), String> {
    let manager_data = manager();
    let v = match manager_data.get(&event) {
        Some(v) => v,
        None => return Err(String::from("invalid event name")),
    };

    match v(data) {
        Ok(v) => (),
        Err(_) => return Err(String::from("event handler has failed to resolve")),
    };

    Ok(())
}

fn print(value: JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    println!("value: {}", value.to_string());
    Ok(())
}
