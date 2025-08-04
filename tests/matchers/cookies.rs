use crate::matchers::{expect_fails_with2, MultiValueMatcherTestSet};
use http::{HeaderMap, HeaderValue};
use httpmock::{MockServer, When};

#[test]
#[cfg(feature = "cookies")]
fn cookie() {
    for (idx, data) in generate_data().attribute.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_not() {
    for (idx, data) in generate_data().attribute_not.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_not(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_exists() {
    for (idx, data) in generate_data().attribute_exists.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_exists(data.expect),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_missing() {
    for (idx, data) in generate_data().attribute_missing.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_missing(data.expect),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_includes() {
    for (idx, data) in generate_data().attribute_includes.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_includes(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_excludes() {
    for (idx, data) in generate_data().attribute_excludes.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_excludes(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_prefix() {
    for (idx, data) in generate_data().attribute_prefix.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_prefix(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_suffix() {
    for (idx, data) in generate_data().attribute_suffix.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_suffix(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_prefix_not() {
    for (idx, data) in generate_data().attribute_prefix_not.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_prefix_not(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_suffix_not() {
    for (idx, data) in generate_data().attribute_suffix_not.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_suffix_not(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_matches() {
    for (idx, data) in generate_data().attribute_matches.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_matches(data.expect.0, data.expect.1),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

#[test]
#[cfg(feature = "cookies")]
fn cookie_count() {
    for (idx, data) in generate_data().attribute_count.iter().enumerate() {
        run_test(
            format!("Running test case with index '{idx}' and test data: {data:?}"),
            |when| when.cookie_count(data.expect.0, data.expect.1, data.expect.2),
            data.actual.clone(),
            data.failure_msg.clone(),
        )
    }
}

fn generate_data() -> MultiValueMatcherTestSet<&'static str, &'static str, usize, &'static str> {
    MultiValueMatcherTestSet::generate("cookie", "Cookie Mismatch", false)
}

fn run_test<F, S>(
    name: S,
    set_expectation: F,
    actual: Vec<(&'static str, &'static str)>,
    error_msg: Option<Vec<&'static str>>,
) where
    F: Fn(When) -> When + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
    S: Into<String>,
{
    println!("{}", name.into());

    let run = || {
        // Arrange
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            set_expectation(when);
            then.status(200);
        });

        // Act
        let mut content = Vec::new();
        for (key, value) in actual {
            if value.contains(' ') || value.contains(';') || value.contains(',') {
                content.push(format!("{}={}", key, value.replace('"', "\\\"")))
            } else {
                content.push(format!("{key}={value}"))
            }
        }

        let value = content.join(";");

        let mut headers = HeaderMap::new();
        headers.insert("cookie", HeaderValue::from_str(&value).unwrap());

        let response = reqwest::blocking::Client::new()
            .get(server.url("/test"))
            .headers(headers)
            .send()
            .unwrap();

        // Assert
        m.assert();
        assert_eq!(response.status(), 200);
    };

    if let Some(err_msg) = error_msg {
        expect_fails_with2(err_msg, run);
    } else {
        run();
    }
}
