use {
    crate::{
        ctx, 
        rpc::{RpcHost, RpcResponse, RpcTrait},
    },
    anyhow::Error,
    astra::{Body, Request, ResponseBuilder, Server},
    serde::Serialize, 
    serde_json::{Map, Value as JsonValue}, 
    std::{any::Any, sync::atomic::Ordering}
};

pub struct JsonRpc {
}

impl JsonRpc {
    pub fn new() -> Self {
        Self {}
    }
}

impl RpcTrait for JsonRpc {
    fn route_message(&self, message: &[&dyn Any]) -> RpcResponse {
        let headers = message[0].downcast_ref::<JsonRpcHeaders>().unwrap();
        let method = headers.get_method(message);
        if let Some(method) = method {
            match method.as_str() {
                "get_height" => self.get_height(),
                "get_block" => self.get_block(message),
                _ => RpcResponse::Error(format!("Method not found: {}", method)),
            }
        } else {
            RpcResponse::Error("No method provided".to_string())
        }
    }

    fn listen(&self, host: RpcHost, port: u16) -> Result<(), Error> {
        info!("Listening on {}:{}", host, port);
        match host {
            RpcHost::Localhost => Server::bind(format!("127.0.0.1:{}", port)),
            RpcHost::Public(ip) => Server::bind(format!("{}:{}", ip, port)),
        }.serve(|_req: Request, _info| {
            //let body = req.extensions().get::<Vec<u8>>().unwrap();
            //println!("{}", String::from_utf8(body.clone()).unwrap());
            ResponseBuilder::new()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new("Hello, world!"))
                .unwrap()
        }).map_err(|e| Error::new(e))
    }
}

// routes
#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub result: JsonValue,
    pub error: Option<JsonValue>,
}

macro_rules! json_obj {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = Map::new();
        $(
            map.insert($key.to_string(), $value);
        )*
        JsonValue::Object(map)
    }};
}

impl JsonRpc {
    fn get_height(&self) -> RpcResponse {
        let response = JsonRpcResponse {
            result: json_obj!(
                "height" => JsonValue::Number(ctx!(block_height).load(Ordering::Relaxed).into()),
                "block_hash" => JsonValue::String(ctx!(curr_top).read().unwrap().to_string())
            ),
            error: None,
        };
        RpcResponse::Success(serde_json::to_vec(&response).expect("Failed to serialize response"))
    }

    fn get_block(&self, message: &[&dyn Any]) -> RpcResponse {
        match self.get_body(message) {
            Some(body) => {
                if let Some(hash) = body.get("hash") {
                    let hash = hash.as_str().unwrap();
                    let response = JsonRpcResponse {
                        result: json_obj!(
                            "hash" => JsonValue::String(hash.to_string()),
                        ),
                        error: None,
                    };
                    RpcResponse::Success(serde_json::to_vec(&response).expect("Failed to serialize response"))
                } else {
                    RpcResponse::Error("No hash provided".to_string())
                }
            }
            None => RpcResponse::Error("No body provided".to_string()),
        }
    }
}

pub struct JsonRpcHeaders {
}

impl JsonRpcHeaders {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_method(&self, _message: &[&dyn Any]) -> Option<String> {
        None
    }
}

impl JsonRpc {
    fn get_body(&self, _params: &[&dyn Any]) -> Option<Map<String, JsonValue>> {
        None
    }
}