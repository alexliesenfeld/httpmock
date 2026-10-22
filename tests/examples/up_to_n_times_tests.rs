use httpmock::prelude::*;

#[test]
fn then_up_to_n_times_test() {
    // Arrange
    let server = MockServer::start();

    let failing = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(503).up_to_n_times(2);
    });

    let healthy = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(200).up_to_n_times(1);
    });

    // Act
    let statuses: Vec<u16> = (0..4)
        .map(|_| reqwest::blocking::get(server.url("/health")).unwrap().status().as_u16())
        .collect();

    // Assert: Both mocks stop matching once their limit is reached, but keep their call counters
    assert_eq!(statuses, vec![503, 503, 200, 404]);
    failing.assert_calls(2);
    healthy.assert_calls(1);
}

#[test]
fn mock_up_to_n_times_test() {
    // Arrange
    let server = MockServer::start();

    let first = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205);
    });
    first.up_to_n_times(2);

    let second = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(500);
    });

    // Act
    let statuses: Vec<u16> = (0..3)
        .map(|_| reqwest::blocking::get(server.url("/health")).unwrap().status().as_u16())
        .collect();

    // Assert
    assert_eq!(statuses, vec![205, 205, 500]);
    first.assert_calls(2);
    second.assert_calls(1);
}

#[test]
fn mock_up_to_n_times_counts_previous_calls_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205);
    });

    for _ in 0..3 {
        let response = reqwest::blocking::get(server.url("/health")).unwrap();
        assert_eq!(response.status(), 205);
    }

    // Act: The mock has already served more calls than the limit allows
    m.up_to_n_times(2);
    let response = reqwest::blocking::get(server.url("/health")).unwrap();

    // Assert
    assert_eq!(response.status(), 404);
    m.assert_calls(3);
}

#[test]
#[should_panic(expected = "The mock stopped matching after reaching its up_to_n_times limit of 2")]
fn assert_calls_on_exhausted_mock_names_the_limit_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205).up_to_n_times(2);
    });

    for _ in 0..3 {
        reqwest::blocking::get(server.url("/health")).unwrap();
    }

    // Act: The third request matched the mock's conditions, but the mock no longer served it
    m.assert_calls(3);
}

#[test]
fn delete_exhausted_mock_test() {
    // Arrange
    let server = MockServer::start();

    let mut m = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205).up_to_n_times(1);
    });
    reqwest::blocking::get(server.url("/health")).unwrap();

    // Act: Delete the exhausted mock, which is still registered
    m.delete();

    // Assert
    let response = reqwest::blocking::get(server.url("/health")).unwrap();
    assert_eq!(response.status(), 404);
}

#[test]
#[should_panic(expected = "up_to_n_times must be greater than 0")]
fn then_up_to_n_times_rejects_zero_test() {
    // Arrange
    let server = MockServer::start();

    // Act
    server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205).up_to_n_times(0);
    });
}

#[test]
#[should_panic(expected = "up_to_n_times must be greater than 0")]
fn mock_up_to_n_times_rejects_zero_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205);
    });

    // Act
    m.up_to_n_times(0);
}

#[test]
#[should_panic(expected = "could not set the up_to_n_times limit on mock")]
fn mock_up_to_n_times_panics_for_deleted_mock_test() {
    // Arrange
    let server = MockServer::start();

    let mut m = server.mock(|when, then| {
        when.method("GET").path("/health");
        then.status(205);
    });
    m.delete();

    // Act
    m.up_to_n_times(1);
}

#[test]
#[cfg(feature = "remote")]
fn remote_up_to_n_times_test() {
    use reqwest::blocking::Client;

    use crate::with_standalone_server;

    // Arrange

    // This starts up a standalone server in the background running on port 5050
    with_standalone_server();
    let server = MockServer::connect("localhost:5050");

    let from_definition = server.mock(|when, then| {
        when.path("/up-to-n-times/definition").body("hello");
        then.status(202).up_to_n_times(1);
    });

    let from_mock = server.mock(|when, then| {
        when.path("/up-to-n-times/mock").body("hello");
        then.status(202);
    });
    from_mock.up_to_n_times(2);

    // Act
    let client = Client::new();
    let send = |path: &str| {
        client
            .post(server.url(path))
            .body("hello")
            .send()
            .unwrap()
            .status()
            .as_u16()
    };
    let definition_statuses: Vec<u16> = (0..2).map(|_| send("/up-to-n-times/definition")).collect();
    let mock_statuses: Vec<u16> = (0..3).map(|_| send("/up-to-n-times/mock")).collect();

    // Assert
    assert_eq!(definition_statuses, vec![202, 404]);
    assert_eq!(mock_statuses, vec![202, 202, 404]);
    from_definition.assert_calls(1);
    from_mock.assert_calls(2);
}

#[test]
#[cfg(feature = "remote")]
#[should_panic(expected = "could not set the up_to_n_times limit on mock")]
fn remote_up_to_n_times_panics_for_deleted_mock_test() {
    use crate::with_standalone_server;

    // Arrange

    // This starts up a standalone server in the background running on port 5050
    with_standalone_server();
    let server = MockServer::connect("localhost:5050");

    let mut m = server.mock(|when, then| {
        when.path("/up-to-n-times/deleted");
        then.status(202);
    });
    m.delete();

    // Act
    m.up_to_n_times(1);
}

#[test]
#[cfg(feature = "remote")]
#[should_panic(expected = "up_to_n_times must be greater than 0")]
fn remote_up_to_n_times_rejects_zero_test() {
    use crate::with_standalone_server;

    // Arrange

    // This starts up a standalone server in the background running on port 5050
    with_standalone_server();
    let server = MockServer::connect("localhost:5050");

    let m = server.mock(|when, then| {
        when.path("/up-to-n-times/zero");
        then.status(202);
    });

    // Act
    m.up_to_n_times(0);
}
