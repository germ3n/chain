use {
    crate::rpc::{RpcResponse, RpcTrait},
    crate::rpc::json::{JsonRpcHeaders, JsonRpcHeadersTrait, JsonRpcResponse, JsonRpcBody, JsonRpcBodyTrait},    
    std::any::Any,
    crate::serialize::json::{SerializableJSON},
    crate::ctx,
    std::sync::atomic::Ordering,
    serde_json::{Map, Value as JsonValue},
};

macro_rules! json_obj {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = Map::new();
        $(
            map.insert($key.to_string(), $value);
        )*
        JsonValue::Object(map)
    }};
}

pub fn get_height(message: &[&dyn Any]) -> RpcResponse {
    RpcResponse::Success(json_obj!(
        "height" => (ctx!(block_height).load(Ordering::Relaxed) as u64).into()
    ).to_string().into())
}

pub fn get_block(message: &[&dyn Any]) -> RpcResponse {
    RpcResponse::Success(vec![])
}