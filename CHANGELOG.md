# Changelog

## v0.4.0

- Removes Darwin V7 version 1.0 support
- Updates some API Responses to be optional to account for unexpected API changes
  No newline at end of file

## v0.5.0

- Bump dependency versions
- Allow selecting an http stack
- Remove the cargo template binary

## v0.6.0

- Bump dependency versions

## v0.7.0

- Bump dependency versions

## v0.7.1

- Bump dependency versions
- Fixed V7 API change for workflow stages.
- Fixed V7 API change for listing dataset items.

## v0.7.2

- Bump dependency versions

## v0.7.3

- Fixed V7 API response structure for listing annotation classes.

## v0.7.4

- Fixed V7 Workflow payload to create workflows with stages.

## v0.8.0

- Added support to update dataset instructions.
- Breaking change: `Dataset::set_workflow_v2` is now `Team::create_workflow` for correctness.

## v0.8.1

- Fix broken crate publish v0.8.0

## v0.9.0

- Breaking change: A new error type, `DarwinV7Error`, has been introduced. All public APIs now use this error type, rather than `anyhow::Error`.

## v0.9.1

- Marking project as no longer maintained.