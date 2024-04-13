use crate::*;

pub struct DataStorageSettings<T> {
    pub layer_keys: Vec<u8>,
    pub recent_capacity: usize,
    pub get_random_function: Option<fn(&Self) -> Option<&T>>,
}

impl<T> DataStorageSettings<T> {
    pub fn new() -> Self {
        DataStorageSettings {
            layer_keys: FIBONACCI.to_vec(),
            recent_capacity: 10,
            get_random_function: None,
        }
    }
}