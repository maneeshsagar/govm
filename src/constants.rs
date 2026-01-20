/// Base URL for Go downloads
pub const GO_DOWNLOAD_BASE: &str = "https://go.dev/dl/";

/// API endpoint for Go version list
pub const GO_VERSION_LIST: &str = "https://go.dev/dl/?mode=json&include=all";

/// List of Go binaries that need shims
pub const GO_BINARIES: &[&str] = &["go", "gofmt"];
