use {
    anyhow::Error,
    serde::{Serialize, Deserialize},
};

pub trait SerializableJSON {
    fn to_json(&self) -> Result<String, Error>;
}

pub trait DeserializableJSON {
    fn from_json(json: &str) -> Result<Self, Error>
    where Self: Sized;
}

impl<T: Serialize> SerializableJSON for T {
    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(anyhow::Error::msg)
    }
}

impl<T: for<'a> Deserialize<'a>> DeserializableJSON for T {
    fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(anyhow::Error::msg)
    }
}

pub fn to_json<T: Serialize>(value: &T) -> Result<String, Error> {
    value.to_json()
}

pub fn from_json<T: for<'a> Deserialize<'a>>(json: &str) -> Result<T, Error>
where T: Sized {
    T::from_json(json)
}