use {
    anyhow::Error,
    serde::{Serialize, Deserialize},
    cutedogs::document,
};

pub trait SerializableJSON {
    fn to_json(&self) -> Result<String, Error>;
}

pub trait DeserializableJSON {
    fn from_json(json: &str) -> Result<Self, Error>
    where Self: Sized;
}

impl<T: Serialize> SerializableJSON for T {
    #[document(summary = "Serializes an object to JSON", returns = "The serialized object")]
    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(Error::new)
    }
}

impl<T: for<'a> Deserialize<'a>> DeserializableJSON for T {
    #[document(summary = "Deserializes an object from JSON", returns = "The deserialized object")]
    fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(Error::new)
    }
}

#[document(summary = "Serializes an object to JSON", returns = "The serialized object")]
pub fn to_json<T: Serialize>(value: &T) -> Result<String, Error> {
    value.to_json()
}

#[document(summary = "Deserializes an object from JSON", returns = "The deserialized object")]
pub fn from_json<T: for<'a> Deserialize<'a>>(json: &str) -> Result<T, Error>
where T: Sized {
    T::from_json(json)
}