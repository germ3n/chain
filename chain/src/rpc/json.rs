use {
    crate::{
        ctx, 
        rpc::{RpcHost, RpcResponse, RpcTrait},
    },
    anyhow::Error,
    astra::{Body, Request, ResponseBuilder, Server},
    serde::Serialize, 
    serde_json::{Map, Value as JsonValue}, 
    std::{any::Any},
    std::sync::RwLock,
    http::HeaderMap,
    crate::rpc::routes::node_rpc,
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
        /*let headers = message[0].downcast_ref::<JsonRpcHeaders>().unwrap();
        let method = headers.
        if let Some(method) = method {
            match method.as_str() {
                "get_height" => node_rpc::get_height(message),
                "get_block" => node_rpc::get_block(message),
                _ => RpcResponse::Error(format!("Method not found: {}", method)),
            }
        } else {
            RpcResponse::Error("No method provided".to_string())
        }*/
        unimplemented!("route_message not implemented")
    }

    fn listen(&self, host: RpcHost, port: u16) -> Result<(), Error> {
        info!("Listening on {}:{}", host, port);
        match host {
            RpcHost::Localhost => Server::bind(format!("127.0.0.1:{}", port)),
            RpcHost::Public(ip) => Server::bind(format!("{}:{}", ip, port)),
        }.serve(|req: Request, _info| {
            //let body = req.extensions().get::<Vec<u8>>().unwrap();
            //println!("{}", String::from_utf8(body.clone()).unwrap());
            match verify_request(req.headers()) {
                Ok(_) => {
                    ResponseBuilder::new()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(Body::new("Hello, world!"))
                    .unwrap()
                }
                Err(e) => {
                    ResponseBuilder::new()
                    .status(400)
                    .header("Content-Type", "application/json")
                    .body(Body::new(e.to_string()))
                    .unwrap()
                }
            }
        }).map_err(|e| Error::new(e))
    }

    fn verify_request(&self, request: &[&dyn Any]) -> Result<(), Error> {
        verify_request(&request[0].downcast_ref::<JsonRpcHeaders>().unwrap().headers)
    }
}

fn verify_request(headers: &HeaderMap) -> Result<(), Error> {
    let method = headers.get("method");
    if let Some(method) = method {
        Ok(())
    } else {
        Err(Error::msg("No method provided"))
    }
}

// routes
#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub result: JsonValue,
    pub error: Option<JsonValue>,
}

pub trait JsonRpcHeadersTrait {
    fn get_auth(&self, _message: &[&dyn Any]) -> Option<String>;
    fn get_key(&self, _message: &[&dyn Any], _key: &str) -> Option<String>;
}

pub struct JsonRpcHeaders {
    pub headers: HeaderMap
}

impl JsonRpcHeadersTrait for JsonRpcHeaders {
    fn get_auth(&self, message: &[&dyn Any]) -> Option<String> {
        self.get_key(message, "X-Authorizations")
    }

    fn get_key(&self, message: &[&dyn Any], key: &str) -> Option<String> {
        match self.headers.get(key) {
            Some(value) => Some(value.to_str().unwrap().to_string()),
            None => None,
        }
    }
}   

pub trait JsonRpcBodyTrait {
    fn new(request: Request) -> Self;
}

pub struct JsonRpcBody {
    //pub body: Request,
    pub json_body: RwLock<Option<Map<String, JsonValue>>>,
}

impl JsonRpcBodyTrait for JsonRpcBody {
    fn new(request: Request) -> Self {
        let body = request.extensions().get::<Vec<u8>>().unwrap();
        let body: Map<String, JsonValue> = serde_json::from_slice(body).unwrap();
        Self {
            //body: request,
            json_body: RwLock::new(Some(body)),
        }
    }
}