use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestStruct {
    pub data: String,
}

#[test]
pub fn test_set_config() {
    let save_config = TestStruct {
        data: "Hello World!".to_string(),
    };
    nihility_config::set_config::<TestStruct>("test_set", &save_config).expect("Failed to set config");
    let read_config = nihility_config::get_config::<TestStruct>("test_set").expect("Failed to get config");
    assert_eq!(save_config, read_config);
}