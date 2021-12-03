use std::convert::TryInto;

use json::JsonValue;
use neon::{prelude::*, result::Throw};

use crate::error::predeclared::QuickSocketError;

pub enum JsonTypes {
    Array,
    Boolean,
    Null,
    Number,
    Object,
    String,
    Unknown,
}

// pub struct JsonParser

pub fn determine_js_type<'b, C>(cx: &mut C, value: &Handle<JsValue>) -> JsonTypes
where
    C: Context<'b>,
{
    if value.is_a::<JsArray, _>(cx) {
        JsonTypes::Array
    } else if value.is_a::<JsBoolean, _>(cx) {
        JsonTypes::Boolean
    } else if value.is_a::<JsNull, _>(cx) {
        JsonTypes::Null
    } else if value.is_a::<JsNumber, _>(cx) {
        JsonTypes::Number
    } else if value.is_a::<JsObject, _>(cx) {
        JsonTypes::Object
    } else if value.is_a::<JsString, _>(cx) {
        JsonTypes::String
    } else {
        JsonTypes::Unknown
    }
}

pub fn determine_json_type(value: &JsonValue) -> JsonTypes {
    if value.is_array() {
        JsonTypes::Array
    } else if value.is_boolean() {
        JsonTypes::Boolean
    } else if value.is_null() {
        JsonTypes::Null
    } else if value.is_number() {
        JsonTypes::Number
    } else if value.is_object() {
        JsonTypes::Object
    } else if value.is_string() {
        JsonTypes::String
    } else {
        JsonTypes::Unknown
    }
}

pub fn parse_js_to_json(
    cx: &mut FunctionContext,
    value: Handle<JsObject>,
) -> Result<json::object::Object, Throw> {
    fn array<'a>(cx: &mut FunctionContext, value: Handle<JsValue>) -> Result<json::Array, Throw> {
        let values: Handle<JsArray> = value.downcast_or_throw(cx)?;
        let values = values.to_vec(cx)?;

        let mut return_value = vec![];

        for v in values {
            match determine_js_type(cx, &v) {
                JsonTypes::Array => {
                    return_value.push(array(cx, v)?.into());
                }
                JsonTypes::Boolean => {
                    return_value.push(boolean(cx, v)?);
                }
                JsonTypes::Null => {
                    return_value.push(null(cx, v)?);
                }
                JsonTypes::Number => {
                    return_value.push(number(cx, v)?.into());
                }
                JsonTypes::Object => {
                    return_value.push(object(cx, v)?.into());
                }
                JsonTypes::String => {
                    return_value.push(string(cx, v)?);
                }
                JsonTypes::Unknown => {
                    return cx.throw_error("json data invalid");
                }
            }
        }

        Ok(return_value)
    }
    fn boolean<'a>(
        cx: &mut FunctionContext,
        value: Handle<JsValue>,
    ) -> Result<json::JsonValue, Throw> {
        let value: Handle<JsBoolean> = value.downcast_or_throw(cx)?;
        let value = value.value(cx);

        Ok(value.into())
    }
    fn null<'a>(
        cx: &mut FunctionContext,
        value: Handle<JsValue>,
    ) -> Result<json::JsonValue, Throw> {
        Ok(JsonValue::Null)
    }
    fn number<'a>(
        cx: &mut FunctionContext,
        value: Handle<JsValue>,
    ) -> Result<json::number::Number, Throw> {
        let value: Handle<JsNumber> = value.downcast_or_throw(cx)?;
        let value = value.value(cx);

        Ok(value.into())
    }
    fn object<'a>(
        cx: &mut FunctionContext,
        value: Handle<JsValue>,
    ) -> Result<json::object::Object, Throw> {
        let master_value: Handle<JsObject> = value.downcast_or_throw(cx)?;
        let keys = master_value.get_own_property_names(cx)?.to_vec(cx)?;

        let mut return_value = json::object::Object::new();

        for key in keys {
            let value = master_value.get(cx, key.clone())?;
            let key: Handle<JsString> = key.downcast_or_throw(cx)?;
            let key = key.value(cx);
            match determine_js_type(cx, &value) {
                JsonTypes::Array => {
                    return_value.insert(key.as_str(), array(cx, value)?.into());
                }
                JsonTypes::Boolean => {
                    return_value.insert(key.as_str(), boolean(cx, value)?);
                }
                JsonTypes::Null => {
                    return_value.insert(key.as_str(), null(cx, value)?);
                }
                JsonTypes::Number => {
                    return_value.insert(key.as_str(), number(cx, value)?.into());
                }
                JsonTypes::Object => {
                    return_value.insert(key.as_str(), object(cx, value)?.into());
                }
                JsonTypes::String => {
                    return_value.insert(key.as_str(), string(cx, value)?);
                }
                JsonTypes::Unknown => {
                    return cx.throw_error("json parse fail");
                }
            }
        }

        Ok(return_value)
    }
    fn string<'a>(
        cx: &mut FunctionContext,
        value: Handle<JsValue>,
    ) -> Result<json::JsonValue, Throw> {
        let value: Handle<JsString> = value.downcast_or_throw(cx)?;
        let value = value.value(cx);

        Ok(value.into())
    }

    Ok(match object(cx, value.upcast()) {
        Ok(v) => v,
        Err(_) => return cx.throw_error("json parse fail"),
    })
}

pub fn parse_json_to_js<'a, C>(
    cx: &mut C,
    value: json::JsonValue,
) -> Result<Handle<'a, JsObject>, Box<dyn std::error::Error>>
where
    C: Context<'a>,
{
    fn array<'b, D>(
        cx: &mut D,
        value: JsonValue,
    ) -> Result<Handle<'b, JsArray>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        let mut return_array: Vec<Handle<JsValue>> = vec![];

        for v in value.members() {
            let val = match determine_json_type(&v) {
                JsonTypes::Array => array(cx, v.to_owned())?.upcast(),
                JsonTypes::Boolean => boolean(cx, v.to_owned())?.upcast(),
                JsonTypes::Null => null(cx, v.to_owned())?.upcast(),
                JsonTypes::Number => number(cx, v.to_owned())?.upcast(),
                JsonTypes::Object => object(cx, v.to_owned())?.upcast(),
                JsonTypes::String => string(cx, v.to_owned())?.upcast(),
                JsonTypes::Unknown => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            return_array.push(val);
        }

        let js_array = JsArray::new(cx, return_array.len().try_into()?);

        for (i, s) in return_array.iter().enumerate() {
            match js_array.set(cx, i as u32, s.to_owned()) {
                Ok(_) => continue,
                Err(_) => return Err(QuickSocketError::JsonParseFail.to_box()),
            };
        }

        Ok(js_array)
    }
    fn boolean<'b, D>(
        cx: &mut D,
        value: JsonValue,
    ) -> Result<Handle<'b, JsBoolean>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        let value = match value.as_bool() {
            Some(v) => v,
            None => return Err(QuickSocketError::JsonParseFail.to_box()),
        };

        Ok(JsBoolean::new(cx, value))
    }
    fn null<'b, D>(
        cx: &mut D,
        _: JsonValue,
    ) -> Result<Handle<'b, JsNull>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        Ok(JsNull::new(cx))
    }
    fn number<'b, D>(
        cx: &mut D,
        value: JsonValue,
    ) -> Result<Handle<'b, JsNumber>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        let v = match value.as_f64() {
            Some(v) => v,
            None => return Err(QuickSocketError::JsonParseFail.to_box()),
        };

        Ok(JsNumber::new::<_, f64>(cx, v))
    }
    fn object<'b, D>(
        cx: &mut D,
        value: JsonValue,
    ) -> Result<Handle<'b, JsObject>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        let jsObject = JsObject::new(cx);

        for (key, value) in value.entries() {
            let value = value.to_owned();
            let value: Handle<JsValue> = match determine_json_type(&value) {
                JsonTypes::Array => array(cx, value)?.upcast(),
                JsonTypes::Boolean => boolean(cx, value)?.upcast(),
                JsonTypes::Null => null(cx, value)?.upcast(),
                JsonTypes::Number => number(cx, value)?.upcast(),
                JsonTypes::Object => object(cx, value)?.upcast(),
                JsonTypes::String => string(cx, value)?.upcast(),
                JsonTypes::Unknown => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            jsObject.set(cx, key, value);
        }

        Ok(jsObject)
    }
    fn string<'b, D>(
        cx: &mut D,
        value: JsonValue,
    ) -> Result<Handle<'b, JsString>, Box<dyn std::error::Error>>
    where
        D: Context<'b>,
    {
        let value = match value.as_str() {
            Some(v) => v.to_string(),
            None => return Err(QuickSocketError::JsonParseFail.to_box()),
        };

        Ok(JsString::new(cx, value))
    }

    let res = object(cx, value)?;

    Ok(res)
}
