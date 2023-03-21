use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAvailableInputsResponse {
    pub devices: Vec<HdmiInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HdmiInput {
    pub id: i32,
    pub locator: String,
    #[serde(deserialize_with = "string_to_bool")]
    pub connected: bool,
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::unknown_variant(s, &["true", "false"])),
    }
}

#[cfg(test)]
mod test {
    use super::HdmiInput;

    #[test]
    fn test_hdmi_input_deserialize() {
        let json_str = r#"{"id":3,"locator":"my locator","connected":"false"}"#;
        let test_struct = serde_json::from_str::<HdmiInput>(&json_str);

        assert!(test_struct.is_ok());

        let result = test_struct.unwrap();
        assert_eq!(result.id, 3);
        assert_eq!(result.locator, "my locator");
        assert_eq!(result.connected, false);
    }
}
