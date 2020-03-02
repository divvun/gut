use std::fs::{read_to_string, write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub fn to_string<T: ?Sized>(t: &T) -> Result<String>
where
    T: Serialize,
{
    toml::to_string(t).context("Serialize error")
}

pub fn from_string<'de, T>(content: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    toml::from_str(content).with_context(|| format!("Deserialize error {:?}", content))
}

pub fn write_to_file<T: ?Sized, P: AsRef<Path>>(path: P, t: &T) -> Result<()>
where
    T: Serialize,
{
    let content = toml::to_string(t).context("Serialize error")?;
    write(path, content.as_str()).context("Cannot write to config file.")
}

pub fn read_file<P: AsRef<Path>, T>(path: P) -> Result<T>
where
    T: DeserializeOwned,
{
    let content = read_to_string(path).context("Cannot read from the config file")?;
    from_string(&content)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestToml {
        s: String,
        d: f64,
        i: i32,
    }

    prop_compose! {
      fn data_strategy() (s: String, d: f64, i: i32) -> TestToml {
        TestToml { s: s, d: d, i: i }
      }
    }

    proptest! {
        #[test]
        fn serde_data(data in data_strategy())  {
            let content = to_string(&data).expect("serialized data");
            let new_data = from_string(&content).expect("deserialized data");
            assert_eq!(data, new_data)
        }
    }
}
