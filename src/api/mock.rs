use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;

use serde::Serialize;
use serde_json::Value;

use crate::api::{Method, Regex};
use crate::server::data::{
    MockDefinition, MockMatcherFunction, MockServerHttpResponse, Pattern, RequestRequirements,
};
use crate::util::Join;
use crate::MockServer;
use std::time::Duration;

/// Represents the primary interface to the mock server.
///
/// # Example
/// ```rust
/// extern crate httpmock;
///
/// use httpmock::Method::{GET};
/// use httpmock::{Mock, MockServer};
///
/// #[test]
/// fn example_test() {
///     // Arrange
///     let mock_server = MockServer::start();
///     let search_mock = Mock::new()
///         .expect_path_contains("/search")
///         .expect_query_param("query", "metallica")
///         .return_status(202)
///         .create_on(&mock_server);
///
///     // Act: Send the HTTP request
///     let response = isahc::get(&format!(
///         "http://{}/search?query=metallica",
///         mock_server.address()
///     )).unwrap();
///
///     // Assert
///     assert_eq!(response.status(), 202);
///     assert_eq!(search_mock.times_called(), 1);
/// }
/// ```
/// Make sure to create the mock using [Mock::create_on](struct.Mock.html#method.create_on)
/// or [Mock::create_on_async](struct.Mock.html#method.create_on_async). This will create the mock on
/// the server. Thereafter, the mock will be served whenever clients send HTTP requests that match
/// all mock requirements.
///
/// The [Mock::create_on](struct.Mock.html#method.create_on) and
/// [Mock::create_on_async](struct.Mock.html#method.create_on_async) methods return a mock reference
/// object that identifies the mock on the server side. The reference can be used to fetch
/// mock related information from the server, such as the number of times it has been called or to
/// explicitly delete the mock from the server
/// (see [MockRef::delete](struct.MockRef.html#method.delete)).
/// Fore more examples, please refer to
/// [this crates test directory](https://github.com/alexliesenfeld/httpmock/blob/master/tests/integration_tests.rs ).
pub struct Mock {
    mock: MockDefinition,
}

/// Represents a reference to the mock object on a [MockServer](struct.MockServer.html).
/// It can be used to spy on the mock and also perform some management operations, such as
/// deleting the mock from the [MockServer](struct.MockServer.html).
///
/// # Example
/// ```rust
/// extern crate httpmock;
///
/// use httpmock::Method::{GET};
/// use httpmock::{Mock, MockServer};
///
/// #[test]
/// fn delete_mock_test() {
///     // Arrange: Create mock server and a mock
///     let mock_server = MockServer::start();
///     let mut mock = Mock::new()
///         .expect_path_contains("/test")
///         .return_status(202)
///         .create_on(&mock_server);
///
///     // Send a first request, then delete the mock from the mock and send another request.
///     let response1 = isahc::get(mock_server.url("/test")).unwrap();
///
///     // Fetch how often this mock has been called from the server until now
///     assert_eq!(search_mock.times_called(), 1);
///     // Delete the mock from the mock server
///     mock.delete();
///
///     let response2 = isahc::get(mock_server.url("/test")).unwrap();
///
///     // Assert that the mock worked for the first request, but not for the second request,
///     // because it was deleted before the second request was sent.
///     assert_eq!(response1.status(), 202);
///     assert_eq!(response2.status(), 404);
/// }
/// ```
pub struct MockRef<'a> {
    id: usize,
    mock_server: &'a MockServer,
}

impl<'a> MockRef<'a> {
    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```rust
    /// extern crate httpmock;
    ///
    /// use httpmock::Method::{GET};
    /// use httpmock::{Mock, MockServer};
    ///
    /// #[test]
    /// fn times_called_test() {
    ///     // Arrange: Create mock server and a mock
    ///     let mock_server = MockServer::start();
    ///     let mut mock = Mock::new()
    ///         .expect_path_contains("/times_called")
    ///         .return_status(200)
    ///         .create_on(&mock_server);
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get(mock_server.url("/times_called")).unwrap();
    ///
    ///     // Fetch how often this mock has been called from the server until now
    ///     assert_eq!(search_mock.times_called(), 1);
    /// }
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (remote) server.
    pub fn times_called(&self) -> usize {
        self.times_called_async().join()
    }

    /// This method returns the number of times a mock has been called at the mock server.
    /// This method is the asynchronous equivalent of
    /// [MockRef::times_called](struct.MockRef.html#method.times_called).
    ///
    /// # Example
    /// ```rust
    /// extern crate httpmock;
    ///
    /// use httpmock::Method::{GET};
    /// use httpmock::{Mock, MockServer};
    ///
    /// #[test]
    /// #[tokio::test]
    /// fn times_called_test() {
    ///     // Arrange: Create mock server and a mock
    ///     let mock_server = MockServer::start_async().await;
    ///     let mut mock = Mock::new()
    ///         .expect_path_contains("/times_called")
    ///         .return_status(200)
    ///         .create_on_async(&mock_server)
    ///         .await;
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get_async(mock_server.url("/times_called")).await.unwrap();
    ///
    ///     // Fetch how often this mock has been called from the server until now
    ///     assert_eq!(search_mock.times_called_async().await, 1);
    /// }
    /// ```
    /// # Panics
    /// This method will panic if there is a problem to communicate with the server.
    pub async fn times_called_async(&self) -> usize {
        let response = self
            .mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        response.call_counter
    }

    /// Deletes this mock from the mock server.
    ///
    /// # Example
    /// ```rust
    /// extern crate httpmock;
    ///
    /// use httpmock::Method::{GET};
    /// use httpmock::{Mock, MockServer};
    ///
    /// #[test]
    /// fn delete_mock_test() {
    ///     // Arrange: Create mock server and a mock
    ///     let mock_server = MockServer::start();
    ///     let mut mock = Mock::new()
    ///         .expect_path_contains("/test")
    ///         .return_status(202)
    ///         .create_on(&mock_server);
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    ///     // Delete the mock from the mock server
    ///     mock.delete();
    ///
    ///     let response2 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    ///     // Assert that the mock worked for the first request, but not for the second request,
    ///     // because it was deleted before the second request was sent.
    ///     assert_eq!(response1.status(), 202);
    ///     assert_eq!(response2.status(), 404);
    /// }
    /// ```
    /// # Panics
    /// This method will panic if there is a problem to communicate with the server.
    pub fn delete(&mut self) {
        self.delete_async().join();
    }

    /// Deletes this mock from the mock server. This method is the asynchronous equivalent of
    /// [MockRef::delete](struct.MockRef.html#method.delete).
    ///
    /// # Example
    /// ```rust
    /// extern crate httpmock;
    ///
    /// use httpmock::Method::{GET};
    /// use httpmock::{Mock, MockServer};
    ///
    /// #[test]
    /// fn delete_mock_test() {
    ///     // Arrange: Create mock server and a mock
    ///     let mock_server = MockServer::start();
    ///     let mut mock = Mock::new()
    ///         .expect_path_contains("/test")
    ///         .return_status(202)
    ///         .create_on(&mock_server);
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    ///     // Delete the mock from the mock server
    ///     mock.delete();
    ///
    ///     let response2 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    ///     // Assert that the mock worked for the first request, but not for the second request,
    ///     // because it was deleted before the second request was sent.
    ///     assert_eq!(response1.status(), 202);
    ///     assert_eq!(response2.status(), 404);
    /// }
    /// ```
    /// # Panics
    /// This method will panic if there is a problem to communicate with the server.
    pub async fn delete_async(&self) {
        self.mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .delete_mock(self.id)
            .await
            .expect("could not delete mock from server");
    }

    /// Returns the address of the mock server this mock is using. By default this is
    /// "localhost:5000" if not set otherwise by the environment variables HTTPMOCK_HOST and
    /// HTTPMOCK_PORT.
    pub fn server_address(&self) -> &SocketAddr {
        self.mock_server.server_adapter.as_ref().unwrap().address()
    }
}

impl Mock {
    /// Creates a new mock that automatically returns HTTP status code 200 if hit by an HTTP call.
    pub fn new() -> Self {
        Mock {
            mock: MockDefinition {
                request: RequestRequirements {
                    method: None,
                    path: None,
                    path_contains: None,
                    headers: None,
                    header_exists: None,
                    body: None,
                    json_body: None,
                    json_body_includes: None,
                    body_contains: None,
                    path_matches: None,
                    body_matches: None,
                    query_param_exists: None,
                    query_param: None,
                    matchers: None,
                },
                response: MockServerHttpResponse {
                    status: 200,
                    headers: None,
                    body: None,
                    duration: None,
                },
            },
        }
    }

    /// Sets the expected path. If the path of an HTTP request at the server is equal to the
    /// provided path, the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `path` - The exact path to match against.
    pub fn expect_path(mut self, path: &str) -> Self {
        self.mock.request.path = Some(path.to_string());
        self
    }

    /// Sets an expected path substring. If the path of an HTTP request at the server contains t,
    /// his substring the request will be considered a match for this mock to respond (given all
    /// other criteria are met).
    /// * `substring` - The substring to match against.
    pub fn expect_path_contains(mut self, substring: &str) -> Self {
        if self.mock.request.path_contains.is_none() {
            self.mock.request.path_contains = Some(Vec::new());
        }

        self.mock
            .request
            .path_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());

        self
    }

    /// Sets an expected path regex. If the path of an HTTP request at the server matches this,
    /// regex the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `regex` - The regex to match against.
    pub fn expect_path_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.path_matches.is_none() {
            self.mock.request.path_matches = Some(Vec::new());
        }

        self.mock
            .request
            .path_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets the expected HTTP method. If the path of an HTTP request at the server matches this regex,
    /// the request will be considered a match for this mock to respond (given all other
    /// criteria are met).
    /// * `method` - The HTTP method to match against.
    pub fn expect_method(mut self, method: Method) -> Self {
        self.mock.request.method = Some(method.to_string());
        self
    }

    /// Sets an expected HTTP header. If one of the headers of an HTTP request at the server matches
    /// the provided header key and value, the request will be considered a match for this mock to
    /// respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The HTTP header value.
    pub fn expect_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(BTreeMap::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets an expected HTTP header to exists. If one of the headers of an HTTP request at the
    /// server matches the provided header name, the request will be considered a match for this
    /// mock to respond (given all other criteria are met).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    pub fn expect_header_exists(mut self, name: &str) -> Self {
        if self.mock.request.header_exists.is_none() {
            self.mock.request.header_exists = Some(Vec::new());
        }

        self.mock
            .request
            .header_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());
        self
    }

    /// Sets the expected HTTP body. If the body of an HTTP request at the server matches the
    /// provided body, the request will be considered a match for this mock to respond
    /// (given all other criteria are met). This is an exact match, so all characters are taken
    /// into account, such as whitespace, tabs, etc.
    ///  * `contents` - The HTTP body to match against.
    pub fn expect_body(mut self, contents: &str) -> Self {
        self.mock.request.body = Some(contents.to_string());
        self
    }

    /// Sets the expected HTTP body JSON string. This method expects a serializable serde object
    /// that will be parsed into JSON. If the body of an HTTP request at the server matches the
    /// body according to the provided JSON object, the request will be considered a match for
    /// this mock to respond (given all other criteria are met).
    ///
    /// This is an exact match, so all characters are taken into account at the server side.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    pub fn expect_json_body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        let serialized_body =
            serde_json::to_string(body).expect("cannot serialize json body to JSON string ");

        let value =
            Value::from_str(&serialized_body).expect("cannot convert JSON string to serde value");

        self.mock.request.json_body = Some(value);
        self
    }

    /// Sets an expected partial HTTP body JSON string.
    ///
    /// If the body of an HTTP request at the server matches the
    /// partial, the request will be considered a match for
    /// this mock to respond (given all other criteria are met).
    ///
    /// * `partial` - The JSON partial.
    ///
    /// # Important
    /// The partial string needs to contain the full JSON object path from the root.
    ///
    /// ## Example
    /// If your application sends the following JSON request data to the mock server
    /// ```json
    /// {
    ///     "parent_attribute" : "Some parent data goes here",
    ///     "child" : {
    ///         "target_attribute" : "Target value",
    ///         "other_attribute" : "Another value"
    ///     }
    /// }
    /// ```
    /// and you only want to make sure that `target_attribute` has the value
    /// `Target value`, you need to provide a partial JSON string to this method, that starts from
    /// the root of the JSON object, but may leave out unimportant values:
    /// ```rust
    /// extern crate httpmock;
    ///
    /// use httpmock::Method::{GET};
    /// use httpmock::{Mock, MockServer};
    ///
    /// #[test]
    /// fn delete_mock_test() {
    ///     // Arrange: Create mock server and a mock
    ///     let mock_server = MockServer::start();
    ///     let mut mock = Mock::new()
    ///         .expect_json_body_partial(r#"
    ///             {
    ///                 "child" : {
    ///                     "target_attribute" : "Target value"
    ///                 }
    ///             }
    ///         "#)
    ///         .return_status(202)
    ///         .create_on(&mock_server);
    ///
    ///     // ...
    /// }
    /// ```
    /// String format and attribute order will be ignored.

    pub fn expect_json_body_partial(mut self, partial: &str) -> Self {
        if self.mock.request.json_body_includes.is_none() {
            self.mock.request.json_body_includes = Some(Vec::new());
        }

        let value = Value::from_str(partial).expect("cannot convert JSON string to serde value");

        self.mock
            .request
            .json_body_includes
            .as_mut()
            .unwrap()
            .push(value);
        self
    }

    /// Sets an expected HTTP body substring. If the body of an HTTP request at the server contains
    /// the provided substring, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `substring` - The substring that will matched against.
    pub fn expect_body_contains(mut self, substring: &str) -> Self {
        if self.mock.request.body_contains.is_none() {
            self.mock.request.body_contains = Some(Vec::new());
        }

        self.mock
            .request
            .body_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());
        self
    }

    /// Sets an expected HTTP body regex. If the body of an HTTP request at the server matches
    /// the provided regex, the request will be considered a match for this mock to respond
    /// (given all other criteria are met).
    /// * `regex` - The regex that will matched against.
    pub fn expect_body_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.body_matches.is_none() {
            self.mock.request.body_matches = Some(Vec::new());
        }

        self.mock
            .request
            .body_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets an expected query parameter. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name and value, the request will be considered a
    /// match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    pub fn expect_query_param(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.query_param.is_none() {
            self.mock.request.query_param = Some(BTreeMap::new());
        }

        self.mock
            .request
            .query_param
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets an expected query parameter name. If the query parameters of an HTTP request at the server
    /// contains the provided query parameter name (not considering the value), the request will be
    /// considered a match for this mock to respond (given all other criteria are met).
    /// * `name` - The query parameter name that will matched against.
    pub fn expect_query_param_exists(mut self, name: &str) -> Self {
        if self.mock.request.query_param_exists.is_none() {
            self.mock.request.query_param_exists = Some(Vec::new());
        }

        self.mock
            .request
            .query_param_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());

        self
    }

    /// Sets a custom matcher for expected HTTP request. If this function returns true, the request
    /// is considered a match and the mock server will respond to the request
    /// (given all other criteria are also met).
    /// * `request_matcher` - The matcher function.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServerRequest, MockServer, Mock};
    ///
    /// #[test]
    /// fn custom_matcher_test() {
    ///     // Arrange
    ///     let mock_server = MockServer::start();
    ///     let m = Mock::new()
    ///         .expect_match(|req: MockServerRequest| {
    ///             req.path.contains("es")
    ///         })
    ///         .return_status(200)
    ///         .create_on(&mock_server);
    ///
    ///     // Act: Send the HTTP request
    ///     let response = isahc::get(mock_server.url("/test")).unwrap();
    ///
    ///     // Assert
    ///     assert_eq!(response.status(), 200);
    ///     assert_eq!(m.times_called(), 1);
    /// }
    /// ```
    pub fn expect_match(mut self, request_matcher: MockMatcherFunction) -> Self {
        if self.mock.request.matchers.is_none() {
            self.mock.request.matchers = Some(Vec::new());
        }

        self.mock
            .request
            .matchers
            .as_mut()
            .unwrap()
            .push(request_matcher);

        self
    }

    /// Sets the HTTP status that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `status` - The HTTP status that the mock server will return.
    pub fn return_status(mut self, status: usize) -> Self {
        self.mock.response.status = status as u16;
        self
    }

    /// Sets the HTTP response body that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `body` - The HTTP response body that the mock server will return.
    pub fn return_body(mut self, body: &str) -> Self {
        self.mock.response.body = Some(body.to_string());
        self
    }

    /// Sets the HTTP response JSON body that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    ///
    /// The provided JSON object needs to be both, a deserializable and
    /// serializable serde object. Note that this method does not set the "Content-Type" header
    /// automatically, so you need to provide one yourself!
    ///
    /// * `body` - The HTTP response body the mock server will return in the form of a JSON string.
    pub fn return_json_body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        let serialized_body =
            serde_json::to_string(body).expect("cannot serialize json body to JSON string ");
        self.mock.response.body = Some(serialized_body);
        self
    }

    /// Sets an HTTP header that the mock will return, if an HTTP request fulfills all of
    /// the mocks requirements.
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    pub fn return_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }

        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets a duration that will delay the mock server response.
    /// * `duration` - The delay.
    pub fn return_with_delay(mut self, duration: Duration) -> Self {
        self.mock.response.duration = Some(duration);
        self
    }

    /// This method creates the mock at the server side and returns a `Mock` object
    /// representing the reference of the created mock at the server.
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub fn create_on<'a>(self, mock_server: &'a MockServer) -> MockRef<'a> {
        self.create_on_async(mock_server).join()
    }

    /// This method creates the mock at the server side and returns a `Mock` object
    /// representing the reference of the created mock at the server. This method
    /// is the asynchronous counterpart of [Mock::create_on](struct.Mock.html#method.create_on).
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub async fn create_on_async<'a>(self, mock_server: &'a MockServer) -> MockRef<'a> {
        let response = mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .create_mock(&self.mock)
            .await
            .expect("Cannot deserialize mock server response");
        MockRef {
            id: response.mock_id,
            mock_server,
        }
    }
}

impl Default for Mock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::Mock;

    /// This test makes sure that a mock has a successful response code (200) by default.
    #[test]
    fn fill_mock_requirements() {
        let mock = Mock::default();
        assert_eq!(mock.mock.response.status, 200);
    }
}
