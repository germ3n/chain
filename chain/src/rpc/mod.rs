pub mod json;
pub mod routes;

use {
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
    Private(IpAddress),
}

pub trait RpcTrait: Send + Sync {
    fn listen(&self, host: RpcHost, port: u16) -> Result<(), Error>;
}

impl std::fmt::Display for RpcHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcHost::Localhost => write!(f, "localhost"),
            RpcHost::Public(ip) => write!(f, "{}", ip),
            RpcHost::Private(ip) => write!(f, "{}", ip),
        }
    }
}