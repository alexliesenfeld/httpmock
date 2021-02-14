#[derive(Clone)]
enum Encoding {
    URL,
    Base64,
}

// A String value type that can additionally hold httpmock-related metadata.
pub struct StringValue {
    value: String,
    encoding: Option<Encoding>,
}

// Every string like type that is convertible to a string can be automatically converted into a
// StringValue.
impl<T: Into<String>> From<T> for StringValue {
    fn from(value: T) -> Self {
        StringValue {
            value: value.into(),
            encoding: None,
        }
    }
}

// This trait implementation makes it possible to convert a borrowed StringValue into a
// new (copied/cloned) instance of StringValue.
impl Into<StringValue> for &StringValue {
    fn into(self) -> StringValue {
        StringValue {
            value: self.value.to_string(),
            encoding: self.encoding.clone(),
        }
    }
}

// ************************************************************************************
// The following methods provide url encoding for StringValue instances.
// ************************************************************************************
pub trait URLEncodedExtension {
    fn url_encoded(&self) -> StringValue;
}

impl<T: ToString> URLEncodedExtension for T {
    fn url_encoded(&self) -> StringValue {
        StringValue {
            value: self.to_string(),
            encoding: Some(Encoding::URL),
        }
    }
}
