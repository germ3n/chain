use {
    crate::rpc::RpcResponse,    
    std::any::Any,  
    crate::ctx,
    std::sync::atomic::Ordering,
    serde_json::{Map, Value as JsonValue},
    crate::structs::block::Block,
    cutedogs::document,
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

#[document(summary = "Gets the height of the blockchain", returns = "The height of the blockchain")]
pub fn get_height(_message: &[&dyn Any]) -> (RpcResponse, u16) {
    let (top_block, height) = {
        let top_block = ctx!(curr_top).read().expect("Failed to read curr_top");
        let height = ctx!(block_height).load(Ordering::Relaxed);
        (top_block.clone(), height)
    };
    
    (RpcResponse::Success(serde_json::to_vec(&json_obj!(
        "height" => height.into(),
        "top_block" => serde_json::to_value(top_block).expect("Failed to serialize hash"),
    )).expect("Failed to serialize response")), 200)
}

#[document(summary = "Gets the genesis block of the blockchain", returns = "The genesis block of the blockchain")]
pub fn get_block(_message: &[&dyn Any]) -> (RpcResponse, u16) {
    (RpcResponse::Success(serde_json::to_vec(&json_obj!(
        "block" => serde_json::to_value(Block::genesis(None)).expect("Failed to serialize block"), // todo: use specified hash algorithm
    )).expect("Failed to serialize block")), 200)
}