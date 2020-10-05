// TODO: Implement memoization for Comparators
pub(crate) trait Transformer<I, O> {
    fn transform(&self, v: &I) -> Result<O, String>;
}

// ************************************************************************************************
// Base64ValueTransformer
// ************************************************************************************************
pub(crate) struct DecodeBase64ValueTransformer {}

impl DecodeBase64ValueTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer<String, String> for DecodeBase64ValueTransformer {
    fn transform(&self, v: &String) -> Result<String, String> {
        base64::decode(v)
            .map(|t| String::from_utf8_lossy(&t.as_slice()).into())
            .map_err(|err| err.to_string())
    }
}

// ************************************************************************************************
// Base64ValueTransformer
// ************************************************************************************************
pub(crate) struct ToLowercaseTransformer {}

impl ToLowercaseTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer<String, String> for ToLowercaseTransformer {
    fn transform(&self, v: &String) -> Result<String, String> {
        Ok(v.to_lowercase())
    }
}
