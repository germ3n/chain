use {
    crate::rpc::{RpcHost, RpcResponse, RpcTrait},
    anyhow::Error,
    astra::{Body, Request, ResponseBuilder, Server, ConnectionInfo},
    std::sync::Arc,
    matchit::{Router},
    std::any::Any,
    crate::rpc::routes::node_rpc,
    cutedogs::document,
};

pub struct JsonRpc {
    router: Arc<Router<fn(&[&dyn Any]) -> (RpcResponse, u16)>>,
}

impl JsonRpc {
    #[document(summary = "Creates a new JSON RPC", returns = "The new JSON RPC")]
    pub fn new() -> Self {
        Self {
            router: Arc::new({
                let mut router = Router::new();
                router.insert("/get-height", node_rpc::get_height as fn(&[&dyn Any]) -> (RpcResponse, u16)).expect("Failed to insert get-height route");
                router.insert("/get-block", node_rpc::get_block as fn(&[&dyn Any]) -> (RpcResponse, u16)).expect("Failed to insert get-block route");
                router
            }),
        }
    }
}

impl RpcTrait for JsonRpc {
    #[document(summary = "Listens for RPC requests", params = {
        "host": "The host to listen on",
        "port": "The port to listen on"
    }, returns = "The result of the listening")]
    fn listen(&self, host: RpcHost, port: u16) -> Result<(), Error> {
        info!("Listening on {}:{}", host, port);
        let router = Arc::clone(&self.router);
        match host {
            RpcHost::Localhost => Server::bind(format!("127.0.0.1:{}", port)),
            RpcHost::Public(ip) => Server::bind(format!("{}:{}", ip, port)),
            RpcHost::Private(ip) => Server::bind(format!("{}:{}", ip, port)),
        }.serve(move |req: Request, info| {
            match verify_request(&req, &info) {
                Ok(_) => {
                    let (response, http_status) = match router.at(req.uri().path()) {
                        Ok(matchit::Match { value, params: _ }) => (value)(&[&req]),
                        Err(e) => (RpcResponse::Error(e.to_string()), 404)
                    };
                    let body_data = match response {
                        RpcResponse::Success(data) => data,
                        RpcResponse::Error(msg) => msg.into_bytes(),
                    };
                    ResponseBuilder::new()
                        .status(http_status)
                        .header("Content-Type", "application/json")
                        .body(Body::new(body_data))
                        .expect("Failed to build response")
                }
                Err(e) => {
                    ResponseBuilder::new()
                        .status(400)
                        .header("Content-Type", "application/json")
                        .body(Body::new(e.to_string()))
                        .expect("Failed to build response")
                }
            }
        }).map_err(|e| Error::from(e))
    }
}

#[document(summary = "Verifies a request", params = {
    "req": "The request to verify",
    "info": "The connection info"
}, returns = "The result of the verification")]
fn verify_request(_req: &Request, _info: &ConnectionInfo) -> Result<(), Error> {
    //unimplemented!("verify_request not implemented")
    Ok(())
}