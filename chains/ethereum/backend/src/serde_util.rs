use crate::rstd::vec::Vec;
use serde::Deserialize;

/// Helper type for deserialize a single value or a vector of values
/// Obs: The order matters, must try to deserialize `Vec<T>` first
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ValueOrArray<T> {
    Array(Vec<T>),
    Value(T),
}

/// Helper for parse/serialize `Vec<T>` to `Vec<T>`, `T` or `None` depending on the number of
/// elements. Must be used with `#[serde(with = "opt_value_or_array")]`
pub mod opt_value_or_array {
    use super::{ValueOrArray, Vec};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// # Errors
    /// Only fails if `T` serialization fails
    pub fn serialize<T, S>(values: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        match values.len() {
            // Empty array is serialized as `None`
            0 => serializer.serialize_none(),
            // Single element is serialized as the element itself
            1 => values[0].serialize(serializer),
            // Multiple elements are serialized as an array
            _ => values.serialize(serializer),
        }
    }

    /// # Errors
    /// Only fails if `T` deserialization fails
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let values = match <Option<ValueOrArray<T>> as Deserialize<'de>>::deserialize(deserializer)?
        {
            Some(ValueOrArray::Array(values)) => values,
            Some(ValueOrArray::Value(value)) => vec![value],
            None => Vec::new(),
        };
        Ok(values)
    }
}
