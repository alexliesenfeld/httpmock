---
title: Request Matchers
description: Using request matchers to specify which requests should respond. TODO
---

This section describes matcher functions that enable developers to implement custom matchers.
These matchers execute user-defined code to determine if a request meets specific criteria.

> **Attention:** Custom matchers are **not available** when connecting to standalone mock servers (e.g., by using one of the `connect` methods, such as [`MockServer::connect`](https://docs.rs/httpmock/0.7.0/httpmock/struct.MockServer.html#method.connect)).

## is_true
{{{docs.when.is_true}}}

## is_false
{{{docs.when.is_false}}}
