use rlua;
use rlua::{Context, FromLua, ToLua};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::fmt::{Display, Formatter};

/// Because you cannot impl an external trait for an external struct.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonWrapperValue(JsonValue);

impl Display for JsonWrapperValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JsonWrapperValue {
    pub fn new(value: JsonValue) -> Self {
        JsonWrapperValue(value)
    }

    pub fn from(value: &JsonValue) -> Self {
        JsonWrapperValue(value.clone())
    }
}

impl From<JsonValue> for JsonWrapperValue {
    fn from(val: JsonValue) -> Self {
        JsonWrapperValue::new(val)
    }
}

impl Into<JsonValue> for JsonWrapperValue {
    fn into(self) -> JsonValue {
        self.0
    }
}

impl<'lua> ToLua<'lua> for JsonWrapperValue {
    fn to_lua(self, lua: Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        let result = match self.into() {
            JsonValue::Null => rlua::Value::Nil,
            JsonValue::String(s) => s.as_str().to_lua(lua)?,
            JsonValue::Number(n) => {
                if let Some(ni) = n.as_i64() {
                    return ni.to_lua(lua);
                }

                (n.as_f64()
                    .ok_or_else(|| rlua::Error::ToLuaConversionError {
                        from: "JsonValue::Number",
                        to: "Value::Number",
                        message: None,
                    })? as f64)
                    .to_lua(lua)?
            }
            JsonValue::Bool(b) => b.to_lua(lua)?,
            JsonValue::Object(o) => {
                let iter = o
                    .into_iter()
                    .map(|(k, v)| (k, JsonWrapperValue::new(v.clone())));
                rlua::Value::Table(lua.create_table_from(iter)?)
            }
            JsonValue::Array(a) => {
                let iter = a.into_iter().map(|it| JsonWrapperValue::new(it));
                rlua::Value::Table(lua.create_table_from(iter.enumerate())?)
            }
        };

        Ok(result)
    }
}

impl<'lua> FromLua<'lua> for JsonWrapperValue {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        let result = match lua_value {
            rlua::Value::Nil => JsonValue::Null,
            rlua::Value::Boolean(b) => JsonValue::Bool(b),
            rlua::Value::LightUserData(_) => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "LightUserData",
                    to: "JsonValue",
                    message: Some("Impossible to convert".to_string()),
                })
            }
            rlua::Value::Integer(i) => JsonValue::from(i),
            rlua::Value::Number(n) => JsonValue::from(n),
            rlua::Value::String(s) => JsonValue::from(s.to_str()?),
            rlua::Value::Table(t) => {
                let mut o = json!({});
                for pair in t.pairs::<rlua::String, rlua::Value>() {
                    let (key, value) = pair?;
                    let key = key.to_str()?;
                    let value = JsonWrapperValue::from_lua(value, lua)?.0;
                    o.as_object_mut().unwrap().insert(key.to_string(), value);
                }
                o
            }
            rlua::Value::Function(_) => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "Function",
                    to: "JsonValue",
                    message: Some("Impossible to convert".to_string()),
                })
            }
            rlua::Value::Thread(_) => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "Thread",
                    to: "JsonValue",
                    message: Some("Impossible to convert".to_string()),
                })
            }
            rlua::Value::UserData(_) => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "UserData",
                    to: "JsonValue",
                    message: Some("Impossible to convert".to_string()),
                })
            }
            rlua::Value::Error(_) => {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "Error",
                    to: "JsonValue",
                    message: Some("Impossible to convert".to_string()),
                })
            }
        };

        return Ok(JsonWrapperValue(result));
    }
}
