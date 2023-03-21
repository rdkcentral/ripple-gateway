use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAvailableInputsResponse {
    pub devices: Vec<HdmiInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HdmiInput {
    pub id: i32,
    pub locator: String,
    pub connected: bool,
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
