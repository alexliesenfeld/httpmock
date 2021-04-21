use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[cfg(feature = "color")]
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{Method, Regex};
use crate::data::{
    ActiveMock, ClosestMatch, HttpMockRequest, MockDefinition, MockMatcherFunction,
    MockServerHttpResponse, Pattern, RequestRequirements,
};
use crate::server::{Diff, DiffResult, Mismatch, Reason, Tokenizer};
use crate::util::{get_test_resource_file_path, read_file, Join};
use crate::MockServer;

/// Represents a reference to the mock object on a [MockServer](struct.MockServer.html).
/// It can be used to spy on the mock and also perform some management operations, such as
/// deleting the mock from the [MockServer](struct.MockServer.html).
///
/// # Example
/// ```
/// // Arrange
/// use httpmock::prelude::*;
///
/// let server = MockServer::start();
///
/// let mut mock = server.mock(|when, then|{
///    when.path("/test");
///    then.status(202);
/// });
///
/// // Send a first request, then delete the mock from the mock and send another request.
/// let response1 = isahc::get(server.url("/test")).unwrap();
///
/// // Fetch how often this mock has been called from the server until now
/// assert_eq!(mock.hits(), 1);
///
/// // Delete the mock from the mock server
/// mock.delete();
///
/// let response2 = isahc::get(server.url("/test")).unwrap();
///
/// // Assert
/// assert_eq!(response1.status(), 202);
/// assert_eq!(response2.status(), 404);
/// ```
pub struct MockRef<'a> {
    // Please find the reason why id is public in
    // https://github.com/alexliesenfeld/httpmock/issues/26.
    pub id: usize,
    pub(crate) server: &'a MockServer,
}

impl<'a> MockRef<'a> {
    pub fn new(id: usize, server: &'a MockServer) -> Self {
        Self { id, server }
    }
    /// This method asserts that the mock server received **exactly one** HTTP request that matched
    /// all the request requirements of this mock.
    ///
    /// **Attention**: If you want to assert more than one request, consider using either
    /// [MockRef::assert_hits](struct.MockRef.html#method.assert_hits) or
    /// [MockRef::hits](struct.MockRef.html#method.hits).
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// isahc::get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock server received exactly one request that matched all
    /// // the request requirements of the mock.
    /// mock.assert();
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn assert(&self) {
        self.assert_async().join()
    }

    /// This method asserts that the mock server received **exactly one** HTTP request that matched
    /// all the request requirements of this mock.
    ///
    /// **Attention**: If you want to assert more than one request, consider using either
    /// [MockRef::assert_hits](struct.MockRef.html#method.assert_hits) or
    /// [MockRef::hits](struct.MockRef.html#method.hits).
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    ///
    ///  async_std::task::block_on(async {
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server.mock_async(|when, then| {
    ///         when.path("/hits");
    ///         then.status(200);
    ///     }).await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock server received exactly one request that matched all
    ///     // the request requirements of the mock.
    ///     mock.assert_async().await;
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn assert_async(&self) {
        self.assert_hits_async(1).await
    }

    /// This method asserts that the mock server received the provided number of HTTP requests which
    /// matched all the request requirements of this mock.
    ///
    /// **Attention**: Consider using the shorthand version
    /// [MockRef::assert](struct.MockRef.html#method.assert) if you want to assert only one hit.
    ///
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    /// use isahc::get;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// get(server.url("/hits")).unwrap();
    /// get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock server received exactly two requests that matched all
    /// // the request requirements of the mock.
    /// mock.assert_hits(2);
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn assert_hits(&self, hits: usize) {
        self.assert_hits_async(hits).join()
    }

    /// This method asserts that the mock server received the provided number of HTTP requests which
    /// matched all the request requirements of this mock.
    ///
    /// **Attention**: Consider using the shorthand version
    /// [MockRef::assert_async](struct.MockRef.html#method.assert_async) if you want to assert only one hit.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    ///
    ///  async_std::task::block_on(async {
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server.mock_async(|when, then| {
    ///         when.path("/hits");
    ///         then.status(200);
    ///     }).await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock server received exactly two requests that matched all
    ///     // the request requirements of the mock.
    ///     mock.assert_hits_async(2).await;
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn assert_hits_async(&self, hits: usize) {
        let active_mock = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        if active_mock.call_counter == hits {
            return;
        }

        if active_mock.call_counter > hits {
            assert_eq!(
                active_mock.call_counter, hits,
                "The number of matching requests was higher than expected (expected {} but was {})",
                hits, active_mock.call_counter
            )
        }

        let closest_match = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .verify(&active_mock.definition.request)
            .await
            .expect("Cannot contact mock server");

        fail_with(active_mock.call_counter, hits, closest_match)
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// isahc::get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock has been called exactly one time
    /// assert_eq!(1, mock.hits());
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn hits(&self) -> usize {
        self.hits_async().join()
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// async_std::task::block_on(async {
    ///     // Arrange: Create mock server and a mock
    ///     use httpmock::prelude::*;
    ///
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server
    ///         .mock_async(|when, then| {
    ///             when.path("/hits");
    ///             then.status(200);
    ///         })
    ///         .await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock was called with all required attributes exactly one time.
    ///     assert_eq!(1, mock.hits_async().await);
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn hits_async(&self) -> usize {
        let response = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        response.call_counter
    }

    /// Deletes the associated mock object from the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then|{
    ///    when.path("/test");
    ///    then.status(202);
    /// });
    ///
    /// // Send a first request, then delete the mock from the mock and send another request.
    /// let response1 = isahc::get(server.url("/test")).unwrap();
    ///
    /// // Fetch how often this mock has been called from the server until now
    /// assert_eq!(mock.hits(), 1);
    ///
    /// // Delete the mock from the mock server
    /// mock.delete();
    ///
    /// let response2 = isahc::get(server.url("/test")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response1.status(), 202);
    /// assert_eq!(response2.status(), 404);
    /// ```
    pub fn delete(&mut self) {
        self.delete_async().join();
    }

    /// Deletes this mock from the mock server. This method is the asynchronous equivalent of
    /// [MockRef::delete](struct.MockRef.html#method.delete).
    ///
    /// # Example
    /// ```
    /// async_std::task::block_on(async {
    ///     // Arrange
    ///     use httpmock::prelude::*;
    ///
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server
    ///       .mock_async(|when, then|{
    ///           when.path("/test");
    ///           then.status(202);
    ///       })
    ///       .await;
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get_async(server.url("/test")).await.unwrap();
    ///
    ///     // Fetch how often this mock has been called from the server until now
    ///     assert_eq!(mock.hits_async().await, 1);
    ///
    ///     // Delete the mock from the mock server
    ///     mock.delete_async().await;
    ///
    ///     let response2 = isahc::get_async(server.url("/test")).await.unwrap();
    ///
    ///     // Assert
    ///     assert_eq!(response1.status(), 202);
    ///     assert_eq!(response2.status(), 404);
    /// });
    /// ```
    pub async fn delete_async(&self) {
        self.server
            .server_adapter
            .as_ref()
            .unwrap()
            .delete_mock(self.id)
            .await
            .expect("could not delete mock from server");
    }

    /// Returns the address of the mock server where the associated mock object is store on.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::prelude::*;
    ///
    /// let server = MockServer::start();
    ///
    /// println!("{}", server.address());
    /// // Will print "127.0.0.1:12345",
    /// // where 12345 is the port that the mock server is running on.
    /// ```
    pub fn server_address(&self) -> &SocketAddr {
        self.server.server_adapter.as_ref().unwrap().address()
    }
}

/// The [MockRefExt](trait.MockRefExt.html) trait extends the [MockRef](struct.MockRef.html)
/// structure with some additional functionality, that is usually not required.
pub trait MockRefExt<'a> {
    /// Creates a new [MockRef](struct.MockRef.html) instance that references an already existing
    /// mock on a [MockServer](struct.MockServer.html). This functionality is usually not required.
    /// You can use it if for you need to recreate [MockRef](struct.MockRef.html) instances
    ///.
    /// * `id` - The ID of the existing mock ot the [MockServer](struct.MockServer.html).
    /// * `mock_server` - The [MockServer](struct.MockServer.html) to which the
    /// [MockRef](struct.MockRef.html) instance will reference.
    ///
    /// # Example
    /// ```
    /// use httpmock::{MockServer, MockRef, MockRefExt};
    /// use isahc::get;
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let mock_ref = server.mock(|when, then| {
    ///     when.path("/test");
    ///     then.status(202);
    /// });
    ///
    /// // Store away the mock ID for later usage and drop the MockRef instance.
    /// let mock_id = mock_ref.id();
    /// drop(mock_ref);
    ///
    /// // Act: Send the HTTP request
    /// let response = get(server.url("/test")).unwrap();
    ///
    /// // Create a new MockRef instance that references the earlier mock at the MockServer.
    /// let mock_ref = MockRef::new(mock_id, &server);
    ///
    /// // Use the recreated MockRef as usual.
    /// mock_ref.assert();
    /// assert_eq!(response.status(), 202);
    /// ```
    /// Refer to [`Issue 26`][https://github.com/alexliesenfeld/httpmock/issues/26] for more
    /// information.
    fn new(id: usize, mock_server: &'a MockServer) -> MockRef<'a>;

    /// Returns the ID that the mock was assigned to on the
    /// [MockServer](struct.MockServer.html).
    fn id(&self) -> usize;
}

impl<'a> MockRefExt<'a> for MockRef<'a> {
    fn new(id: usize, mock_server: &'a MockServer) -> MockRef<'a> {
        MockRef {
            id,
            server: mock_server,
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}

fn create_reason_output(reason: &Reason) -> String {
    let mut output = String::new();
    let offsets = match reason.best_match {
        true => ("\t".repeat(5), "\t".repeat(2)),
        false => ("\t".repeat(1), "\t".repeat(2)),
    };
    let actual_text = match reason.best_match {
        true => "Actual (closest match):",
        false => "Actual:",
    };
    output.push_str(&format!(
        "Expected:{}[{}]\t\t{}\n",
        offsets.0, reason.comparison, &reason.expected
    ));
    output.push_str(&format!(
        "{}{}{}\t{}\n",
        actual_text,
        offsets.1,
        " ".repeat(reason.comparison.len() + 7),
        &reason.actual
    ));
    output
}

fn create_diff_result_output(dd: &DiffResult) -> String {
    let mut output = String::new();
    output.push_str("Diff:");
    if dd.differences.is_empty() {
        output.push_str("<empty>");
    }
    output.push_str("\n");

    dd.differences.iter().for_each(|d| {
        match d {
            Diff::Same(e) => {
                output.push_str(&format!("   | {}", e));
            }
            Diff::Add(e) => {
                #[cfg(feature = "color")]
                output.push_str(&format!("+++| {}", e).green().to_string());
                #[cfg(not(feature = "color"))]
                output.push_str(&format!("+++| {}", e));
            }
            Diff::Rem(e) => {
                #[cfg(feature = "color")]
                output.push_str(&format!("---| {}", e).red().to_string());
                #[cfg(not(feature = "color"))]
                output.push_str(&format!("---| {}", e));
            }
        }
        output.push_str("\n")
    });
    output.push_str("\n");
    output
}

fn create_mismatch_output(idx: usize, mm: &Mismatch) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} : {}", idx + 1, &mm.title));
    output.push_str("\n");
    output.push_str(&"-".repeat(90));
    output.push_str("\n");

    mm.reason
        .as_ref()
        .map(|reason| output.push_str(&create_reason_output(reason)));

    mm.diff
        .as_ref()
        .map(|diff_result| output.push_str(&create_diff_result_output(diff_result)));

    output.push_str("\n");
    output
}

fn fail_with(actual_hits: usize, expected_hits: usize, closest_match: Option<ClosestMatch>) {
    match closest_match {
        None => assert!(false, "No request has been received by the mock server."),
        Some(closest_match) => {
            let mut output = String::new();
            output.push_str(&format!(
                "{} of {} expected requests matched the mock specification, .\n",
                actual_hits, expected_hits
            ));
            output.push_str(&format!(
                "Here is a comparison with the most similar non-matching request (request number {}): \n\n",
                closest_match.request_index + 1
            ));

            for (idx, mm) in closest_match.mismatches.iter().enumerate() {
                output.push_str(&create_mismatch_output(idx, &mm));
            }

            closest_match.mismatches.first().map(|mismatch| {
                mismatch
                    .reason
                    .as_ref()
                    .map(|reason| assert_eq!(reason.expected, reason.actual, "{}", output))
            });

            assert!(false, output)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::api::mock::fail_with;
    use crate::data::{ClosestMatch, HttpMockRequest};
    use crate::server::{Diff, DiffResult, Mismatch, Reason, Tokenizer};

    #[test]
    #[should_panic(expected = "1 : This is a title\n\
    ------------------------------------------------------------------------------------------\n\
    Expected:	[equals]		/toast\n\
    Actual:		             	/test\n\
    Diff:\n   | t\n---| e\n+++| oa\n   | st")]
    fn fail_with_message_test() {
        // Arrange
        let closest_match = ClosestMatch {
            request: HttpMockRequest {
                path: "/test".to_string(),
                method: "GET".to_string(),
                headers: None,
                query_params: None,
                body: None,
            },
            request_index: 0,
            mismatches: vec![Mismatch {
                title: "This is a title".to_string(),
                reason: Some(Reason {
                    expected: "/toast".to_string(),
                    actual: "/test".to_string(),
                    comparison: "equals".to_string(),
                    best_match: false,
                }),
                diff: Some(DiffResult {
                    differences: vec![
                        Diff::Same(String::from("t")),
                        Diff::Rem(String::from("e")),
                        Diff::Add(String::from("oa")),
                        Diff::Same(String::from("st")),
                    ],
                    distance: 5,
                    tokenizer: Tokenizer::Line,
                }),
            }],
        };

        // Act
        fail_with(1, 2, Some(closest_match));

        // Assert
        // see "should panic" annotation
    }
}
