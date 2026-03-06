use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestStruct {
    pub data: String,
}

#[test]
pub fn test_get_config() {
    let config = nihility_config::get_config::<TestStruct>("test").unwrap();
    assert_eq!(config, TestStruct::default());
}