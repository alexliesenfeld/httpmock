// TODO: Implement memoization for Comparators
pub(crate) trait ValueDecoder<I, O> {
    fn encode(&self, v: &I) -> O;
    fn decode(&self, v: &I) -> Result<O, String>;
}

// ************************************************************************************************
// Base64ValueTransformer
// ************************************************************************************************
pub(crate) struct Base64ValueDecoder {}

impl ValueDecoder<String, String> for Base64ValueDecoder {
    fn encode(&self, req: &String) -> String {
        base64::encode(req)
    }

    fn decode(&self, v: &String) -> Result<String, String> {
        base64::decode(v)
            .map(|t| String::from_utf8_lossy(&t.as_slice()).to_string())
            .map_err(|r| r.to_string())
    }
}
