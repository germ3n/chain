pub mod json;
pub mod routes;

use {
    std::any::Any,
    anyhow::Error,
    crate::net::IpAddress,
};

pub enum RpcResponse {
    Success(Vec<u8>),
    Error(String),
}

pub enum RpcHost {
    Localhost,
    Public(IpAddress),
}

pub trait RpcTrait: Send + Sync {
    fn verify_request(&self, request: &[&dyn Any]) -> Result<(), Error>;
    fn route_message(&self, message: &[&dyn Any]) -> RpcResponse;
    fn listen(&self, host: RpcHost, port: u16) -> Result<(), Error>;
}

impl std::fmt::Display for RpcHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcHost::Localhost => write!(f, "localhost"),
            RpcHost::Public(ip) => write!(f, "{}", ip),
        }
    }
}