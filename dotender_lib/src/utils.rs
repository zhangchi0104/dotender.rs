use crate::types::Error;

pub fn split_mapping(mapping_raw: &str) -> Result<(&str, &str), Error> {
    mapping_raw
        .split_once(':')
        .ok_or(Error::FileMappingError(mapping_raw))
}
