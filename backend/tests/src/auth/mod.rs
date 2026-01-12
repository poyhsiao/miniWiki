// Temporarily disable auth test modules to improve compilation performance.
// These tests will be re-enabled via the auth-tests feature flag.
//
// TODO: Re-enable auth test modules when backend integration is set up.
// See tracking issue: https://github.com/poyhsiao/miniWiki/issues/XXX
//
// Feature flag to enable: cargo test --features auth-tests

#[cfg(feature = "auth-tests")]
pub mod register_test;
#[cfg(feature = "auth-tests")]
pub mod login_test;
#[cfg(feature = "auth-tests")]
pub mod password_reset_test;