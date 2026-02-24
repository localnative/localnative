use serde::{de, ser, Deserialize, Serialize};

use crate::{rpc::RpcError, ProcessError};

#[derive(Serialize, Deserialize)]
struct Error {
    message: String,
}

macro_rules! impl_error_serialize_deserialize {
    ($($error_type:ty => $serialized_err:ident),*) => {
        $(
            impl Serialize for $error_type {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ser::Serializer,
                {
                    let message = self.to_string();
                    Error { message }.serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for $error_type {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: de::Deserializer<'de>,
                {
                    let error = Error::deserialize(deserializer)?;
                    Ok(Self::$serialized_err(error.message))
                }
            }
        )*
    };
}

impl_error_serialize_deserialize!(RpcError => SerializedErr, ProcessError => SerializedErr);
