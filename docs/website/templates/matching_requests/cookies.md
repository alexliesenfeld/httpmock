---
title: Cookie
description: Using request matchers to specify which requests should respond. TODO
---

This section describes matcher functions designed to target and match cookies headers in incoming HTTP requests.

> **Attention:** To use these matchers, enable the cookies feature by adding `--features=cookies` to your Cargo command. For example: `cargo test --features=cookies`.

## cookie
{{{docs.when.cookie}}}

## cookie_not
{{{docs.when.cookie_not}}}

## cookie_exists
{{{docs.when.cookie_exists}}}

## cookie_missing
{{{docs.when.cookie_missing}}}

## cookie_includes
{{{docs.when.cookie_includes}}}

## cookie_excludes
{{{docs.when.cookie_excludes}}}

## cookie_prefix
{{{docs.when.cookie_prefix}}}

## cookie_prefix_not
{{{docs.when.cookie_prefix_not}}}

## cookie_suffix
{{{docs.when.cookie_suffix}}}

## cookie_suffix_not
{{{docs.when.cookie_suffix_not}}}

## cookie_matches
{{{docs.when.cookie_matches}}}

## cookie_count
{{{docs.when.cookie_count}}}
