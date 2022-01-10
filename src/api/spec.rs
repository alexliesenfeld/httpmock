use crate::common::data::{
    MockMatcherFunction, MockServerHttpResponse, Pattern, RequestRequirements,
};
use crate::common::util::{get_test_resource_file_path, read_file, update_cell};
use crate::{Method, Regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

/// A type that allows the specification of HTTP request values.
pub struct When {
    pub(crate) expectations: Rc<Cell<RequestRequirements>>,
}

impl When {
    /// Sets the mock server to respond to any incoming request.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.any_request();
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(server.url("/anyPath")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn any_request(self) -> Self {
        // This method does nothing. It only exists to make it very explicit that
        // the mock server will respond to any request. This is the default at this time, but
        // may change in the future.
        self
    }

    /// Sets the expected HTTP method.
    ///
    /// * `method` - The HTTP method (a [Method](enum.Method.html) or a `String`).
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.method(GET);
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(server.url("/")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn method(mut self, method: impl Into<Method>) -> Self {
        update_cell(&self.expectations, |e| {
            e.method = Some(method.into().to_string())
        });
        self
    }

    /// Sets the expected URL path.
    /// * `path` - The URL path.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.path_contains("/test");
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(server.url("/test")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn path(mut self, path: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            e.path = Some(path.into());
        });
        self
    }

    /// Sets an substring that the URL path needs to contain.
    /// * `substring` - The substring to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.path_contains("es");
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(server.url("/test")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn path_contains(mut self, substring: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.path_contains.is_none() {
                e.path_contains = Some(Vec::new());
            }
            e.path_contains.as_mut().unwrap().push(substring.into());
        });
        self
    }

    /// Sets a regex that the URL path needs to match.
    /// * `regex` - The regex to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.path_matches(Regex::new("le$").unwrap());
    ///     then.status(200);
    /// });
    ///
    /// isahc::get(server.url("/example")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn path_matches(mut self, regex: impl Into<Regex>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.path_matches.is_none() {
                e.path_matches = Some(Vec::new());
            }
            e.path_matches
                .as_mut()
                .unwrap()
                .push(Pattern::from_regex(regex.into()));
        });
        self
    }

    /// Sets a query parameter that needs to be provided.
    ///
    /// Attention!: The request query keys and values are implicitly *allowed, but is not required*
    /// to be urlencoded! The value you pass here, however, must be in plain text (i.e. not encoded)!
    ///
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::prelude::*;
    ///
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.query_param("query", "Metallica");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// get(server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// ```
    pub fn query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.query_param.is_none() {
                e.query_param = Some(Vec::new());
            }
            e.query_param
                .as_mut()
                .unwrap()
                .push((name.into(), value.into()));
        });
        self
    }

    /// Sets a query parameter that needs to exist in an HTTP request.
    ///
    /// Attention!: The request query key is implicitly *allowed, but is not required* to be
    /// urlencoded! The value you pass here, however, must be in plain text (i.e. not encoded)!
    ///
    /// * `name` - The query parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::prelude::*;
    ///
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then| {
    ///     when.query_param_exists("query");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// get(server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// ```
    pub fn query_param_exists(mut self, name: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.query_param_exists.is_none() {
                e.query_param_exists = Some(Vec::new());
            }
            e.query_param_exists.as_mut().unwrap().push(name.into());
        });
        self
    }

    /// Sets a requirement for a tuple in an x-www-form-urlencoded request body.
    /// Please refer to https://url.spec.whatwg.org/#application/x-www-form-urlencoded for more
    /// information.
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then| {
    ///    when.method(POST)
    ///        .path("/example")
    ///        .header("content-type", "application/x-www-form-urlencoded")
    ///        .x_www_form_urlencoded_tuple("name", "Peter Griffin")
    ///        .x_www_form_urlencoded_tuple("town", "Quahog");
    ///    then.status(202);
    /// });
    ///
    /// let response = Request::post(server.url("/example"))
    ///    .header("content-type", "application/x-www-form-urlencoded")
    ///    .body("name=Peter%20Griffin&town=Quahog")
    ///    .unwrap()
    ///    .send()
    ///    .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 202);
    /// ```
    pub fn x_www_form_urlencoded_tuple(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        update_cell(&self.expectations, |e| {
            if e.x_www_form_urlencoded.is_none() {
                e.x_www_form_urlencoded = Some(Vec::new());
            }
            e.x_www_form_urlencoded
                .as_mut()
                .unwrap()
                .push((key.into(), value.into()));
        });
        self
    }

    /// Sets a requirement for a tuple key in an x-www-form-urlencoded request body.
    /// Please refer to https://url.spec.whatwg.org/#application/x-www-form-urlencoded for more
    /// information.
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then| {
    ///    when.method(POST)
    ///        .path("/example")
    ///        .header("content-type", "application/x-www-form-urlencoded")
    ///        .x_www_form_urlencoded_key_exists("name")
    ///        .x_www_form_urlencoded_key_exists("town");
    ///    then.status(202);
    /// });
    ///
    /// let response = Request::post(server.url("/example"))
    ///    .header("content-type", "application/x-www-form-urlencoded")
    ///    .body("name=Peter%20Griffin&town=Quahog")
    ///    .unwrap()
    ///    .send()
    ///    .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 202);
    /// ```
    pub fn x_www_form_urlencoded_key_exists(mut self, key: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.x_www_form_urlencoded_key_exists.is_none() {
                e.x_www_form_urlencoded_key_exists = Some(Vec::new());
            }
            e.x_www_form_urlencoded_key_exists
                .as_mut()
                .unwrap()
                .push(key.into());
        });
        self
    }

    /// Sets the required HTTP request body content.
    ///
    /// * `body` - The required HTTP request body.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.body("The Great Gatsby");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .body("The Great Gatsby")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn body(mut self, body: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            e.body = Some(body.into());
        });
        self
    }

    /// Sets a [Regex](type.Regex.html) for the expected HTTP body.
    ///
    /// * `regex` - The regex that the HTTP request body will matched against.
    ///
    /// ```
    /// use isahc::{prelude::*, Request};
    /// use httpmock::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.method(POST)
    ///         .path("/books")
    ///         .body_matches(Regex::new("Fellowship").unwrap());
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request
    /// let response = Request::post(server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn body_matches(mut self, regex: impl Into<Regex>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.body_matches.is_none() {
                e.body_matches = Some(Vec::new());
            }
            e.body_matches
                .as_mut()
                .unwrap()
                .push(Pattern::from_regex(regex.into()));
        });
        self
    }

    /// Sets the expected HTTP body substring.
    ///
    /// * `substring` - The substring that will matched against.
    ///
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/books")
    ///         .body_contains("Ring");
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request
    /// let response = Request::post(server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn body_contains(mut self, substring: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.body_contains.is_none() {
                e.body_contains = Some(Vec::new());
            }
            e.body_contains.as_mut().unwrap().push(substring.into());
        });
        self
    }

    /// Sets the expected JSON body. This method expects a [serde_json::Value](../serde_json/enum.Value.html)
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the `content-type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::prelude::*;
    /// use serde_json::json;
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/user")
    ///         .header("content-type", "application/json")
    ///         .json_body(json!({ "name": "Hans" }));
    ///     then.status(201);
    /// });
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/user", server.address()))
    ///     .header("content-type", "application/json")
    ///     .body(json!({ "name": "Hans" }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn json_body(mut self, value: impl Into<serde_json::Value>) -> Self {
        update_cell(&self.expectations, |e| {
            e.json_body = Some(value.into());
        });
        self
    }

    /// Sets the expected JSON body. This method expects a serializable serde object
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the "content-type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::prelude::*;
    /// use serde_json::json;
    /// use isahc::{prelude::*, Request};
    ///
    /// // This is a temporary type that we will use for this test
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/user")
    ///         .header("content-type", "application/json")
    ///         .json_body_obj(&TestUser {
    ///             name: String::from("Fred"),
    ///         });
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/user", server.address()))
    ///     .header("content-type", "application/json")
    ///     .body(json!(&TestUser {
    ///         name: "Fred".to_string()
    ///     }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn json_body_obj<'a, T>(self, body: &T) -> Self
    where
        T: Serialize + Deserialize<'a>,
    {
        let json_value = serde_json::to_value(body).expect("Cannot serialize json body to JSON");
        self.json_body(json_value)
    }

    /// Sets the expected partial JSON body.
    ///
    /// **Attention: The partial string needs to be a valid JSON string. It must contain
    /// the full object hierarchy from the original JSON object but can leave out irrelevant
    /// attributes (see example).**
    ///
    /// Note that this method does not set the `content-type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// String format and attribute order are irrelevant.
    ///
    /// * `partial_body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ## Example
    /// Suppose your application sends the following JSON request body:
    /// ```json
    /// {
    ///     "parent_attribute" : "Some parent data goes here",
    ///     "child" : {
    ///         "target_attribute" : "Example",
    ///         "other_attribute" : "Another value"
    ///     }
    /// }
    /// ```
    /// If we only want to verify that `target_attribute` has value `Example` without the need
    /// to provive a full JSON object, we can use this method as follows:
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then|{
    ///     when.json_body_partial(r#"
    ///         {
    ///             "child" : {
    ///                 "target_attribute" : "Example"
    ///             }
    ///          }
    ///     "#);
    ///     then.status(200);
    /// });
    /// ```
    /// Please note that the JSON partial contains the full object hierachy, i.e. it needs to start
    /// from the root! It leaves out irrelevant attributes, however (`parent_attribute`
    /// and `child.other_attribute`).
    pub fn json_body_partial(mut self, partial: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.json_body_includes.is_none() {
                e.json_body_includes = Some(Vec::new());
            }
            let value = Value::from_str(&partial.into())
                .expect("cannot convert JSON string to serde value");
            e.json_body_includes.as_mut().unwrap().push(value);
        });
        self
    }

    /// Sets the expected HTTP header.
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The header value.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.header("Authorization", "token 1234567890");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.headers.is_none() {
                e.headers = Some(Vec::new());
            }
            e.headers
                .as_mut()
                .unwrap()
                .push((name.into(), value.into()));
        });
        self
    }

    /// Sets the requirement that the HTTP request needs to contain a specific header
    /// (value is unchecked, refer to [Mock::expect_header](struct.Mock.html#method.expect_header)).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.header_exists("Authorization");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn header_exists(mut self, name: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.header_exists.is_none() {
                e.header_exists = Some(Vec::new());
            }
            e.header_exists.as_mut().unwrap().push(name.into());
        });
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    /// **Attention**: Cookie names are **case-sensitive**.
    ///
    /// * `name` - The cookie name.
    /// * `value` - The expected cookie value.
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.cookie("SESSIONID", "1234567890");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn cookie(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.cookies.is_none() {
                e.cookies = Some(Vec::new());
            }
            e.cookies
                .as_mut()
                .unwrap()
                .push((name.into(), value.into()));
        });
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    /// **Attention**: Cookie names are **case-sensitive**.
    ///
    /// * `name` - The cookie name
    ///
    /// # Example
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then|{
    ///     when.cookie_exists("SESSIONID");
    ///     then.status(200);
    /// });
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn cookie_exists(mut self, name: impl Into<String>) -> Self {
        update_cell(&self.expectations, |e| {
            if e.cookie_exists.is_none() {
                e.cookie_exists = Some(Vec::new());
            }
            e.cookie_exists.as_mut().unwrap().push(name.into());
        });
        self
    }
    /// Sets a custom matcher for expected HTTP request. If this function returns true, the request
    /// is considered a match and the mock server will respond to the request
    /// (given all other criteria are also met).
    /// * `request_matcher` - The matcher function.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///    when.matches(|req: &HttpMockRequest| {
    ///         req.path.contains("es")
    ///    });
    ///    then.status(200);
    /// });
    ///
    /// // Act: Send the HTTP request
    /// let response = isahc::get(server.url("/test")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn matches(mut self, matcher: MockMatcherFunction) -> Self {
        update_cell(&self.expectations, |e| {
            if e.matchers.is_none() {
                e.matchers = Some(Vec::new());
            }
            e.matchers.as_mut().unwrap().push(matcher);
        });
        self
    }
}

/// A type that allows the specification of HTTP response values.
pub struct Then {
    pub(crate) response_template: Rc<Cell<MockServerHttpResponse>>,
}

impl Then {
    /// Sets the HTTP response code that will be returned by the mock server.
    ///
    /// * `status` - The status code.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::prelude::*;
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/hello");
    ///     then.status(200);
    /// });
    ///
    /// // Act
    /// let response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn status(mut self, status: u16) -> Self {
        update_cell(&self.response_template, |r| {
            r.status = Some(status);
        });
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `body` - The response body content.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then| {
    ///     when.path("/hello");
    ///     then.status(200)
    ///         .body("ohi!");
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// ```
    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        update_cell(&self.response_template, |r| {
            r.body = Some(body.as_ref().to_vec());
        });
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `body` - The response body content.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/hello");
    ///     then.status(200)
    ///         .body_from_file("tests/resources/simple_body.txt");
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// ```
    pub fn body_from_file(mut self, resource_file_path: impl Into<String>) -> Self {
        let resource_file_path = resource_file_path.into();
        let path = Path::new(&resource_file_path);
        let absolute_path = match path.is_absolute() {
            true => path.to_path_buf(),
            false => get_test_resource_file_path(&resource_file_path).expect(&format!(
                "Cannot create absolute path from string '{}'",
                &resource_file_path
            )),
        };
        let content = read_file(&absolute_path).expect(&format!(
            "Cannot read from file {}",
            absolute_path.to_str().expect("Invalid OS path")
        ));
        self.body(content)
    }

    /// Sets the JSON body for the HTTP response that will be returned by the mock server.
    ///
    /// The provided JSON object needs to be both, a deserializable and serializable serde object.
    ///
    /// Note that this method does not set the "content-type" header automatically, so you need
    /// to provide one yourself!
    ///
    /// * `body` -  The HTTP response body the mock server will return in the form of a
    ///             serde_json::Value object.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// use httpmock::prelude::*;
    /// use serde_json::{Value, json};
    /// use isahc::ResponseExt;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/user");
    ///     then.status(200)
    ///         .header("content-type", "application/json")
    ///         .json_body(json!({ "name": "Hans" }));
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/user")).unwrap();
    ///
    /// let user: Value =
    ///     serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(user.as_object().unwrap().get("name").unwrap(), "Hans");
    /// ```
    pub fn json_body(mut self, body: impl Into<Value>) -> Self {
        update_cell(&self.response_template, |r| {
            r.body = Some(body.into().to_string().into_bytes());
        });
        self
    }

    /// Sets the JSON body that will be returned by the mock server.
    /// This method expects a serializable serde object that will be serialized/deserialized
    /// to/from a JSON string.
    ///
    /// Note that this method does not set the "content-type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::prelude::*;
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // This is a temporary type that we will use for this example
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then| {
    ///     when.path("/user");
    ///     then.status(200)
    ///         .header("content-type", "application/json")
    ///         .json_body_obj(&TestUser {
    ///             name: String::from("Hans"),
    ///         });
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/user")).unwrap();
    ///
    /// let user: TestUser =
    ///     serde_json::from_str(&response.text().unwrap()).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(user.name, "Hans");
    /// ```
    pub fn json_body_obj<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        let json_body =
            serde_json::to_value(body).expect("cannot serialize json body to JSON string ");
        self.json_body(json_body)
    }

    /// Sets an HTTP header that the mock server will return.
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// // Arrange
    /// use httpmock::prelude::*;
    /// use serde_json::Value;
    /// use isahc::ResponseExt;
    ///
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = server.mock(|when, then|{
    ///     when.path("/");
    ///     then.status(200)
    ///         .header("Expires", "Wed, 21 Oct 2050 07:28:00 GMT")
    ///         .header("link", format!("<{}>; rel=next", server.base_url()));
    /// });
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        update_cell(&self.response_template, |r| {
            if r.headers.is_none() {
                r.headers = Some(Vec::new());
            }
            r.headers
                .as_mut()
                .unwrap()
                .push((name.into(), value.into()));
        });
        self
    }

    /// Sets a duration that will delay the mock server response.
    ///
    /// * `duration` - The delay.
    ///
    /// ```
    /// // Arrange
    /// use std::time::{SystemTime, Duration};
    /// use httpmock::prelude::*;
    ///
    /// let _ = env_logger::try_init();
    /// let start_time = SystemTime::now();
    /// let three_seconds = Duration::from_secs(3);
    /// let server = MockServer::start();
    ///
    /// let mock = server.mock(|when, then| {
    ///     when.path("/delay");
    ///     then.status(200)
    ///         .delay(three_seconds);
    /// });
    ///
    /// // Act
    /// let response = isahc::get(server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// mock.assert();
    /// assert_eq!(start_time.elapsed().unwrap() > three_seconds, true);
    /// ```
    pub fn delay(mut self, duration: impl Into<Duration>) -> Self {
        update_cell(&self.response_template, |r| {
            r.delay = Some(duration.into());
        });
        self
    }
}
