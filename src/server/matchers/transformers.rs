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


#[cfg(test)]
mod test {
    use crate::server::matchers::transformers::{DecodeBase64ValueTransformer, Transformer, ToLowercaseTransformer};

    #[test]
    fn base64_decode_transformer() {
        // Arrange
        let transformer = DecodeBase64ValueTransformer::new();

        // Act
        let result = transformer.transform(&"dGVzdA==".to_string());

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), "test".to_string());
    }


    #[test]
    fn base64_decode_transformer_error() {
        // Arrange
        let transformer = DecodeBase64ValueTransformer::new();

        // Act
        let result = transformer.transform(&"xÿ".to_string());

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn to_lowercase_transformer() {
        // Arrange
        let transformer = ToLowercaseTransformer::new();

        // Act
        let result = transformer.transform(&"HeLlO".to_string());

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), "hello".to_string());
    }

}

