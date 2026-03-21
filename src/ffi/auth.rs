//! FFI Bindings to gopher-auth
//!
//! Provides safe Rust bindings to the gopher-auth native library for JWT validation.
//!
//! # Example
//!
//! ```ignore
//! use gopher_orch::ffi::auth::GopherAuthClient;
//!
//! let client = GopherAuthClient::new(
//!     "https://auth.example.com/.well-known/jwks.json",
//!     "https://auth.example.com"
//! )?;
//!
//! let result = client.validate_token("eyJ...", 60);
//! if result.valid {
//!     println!("Token is valid!");
//! }
//! ```

use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

use libloading::{Library, Symbol};

use crate::error::Error;

/// Result of token validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the token is valid.
    pub valid: bool,
    /// Error code (0 for success).
    pub error_code: i32,
    /// Error message if validation failed.
    pub error_message: Option<String>,
}

impl ValidationResult {
    /// Create a successful validation result.
    pub fn success() -> Self {
        Self {
            valid: true,
            error_code: 0,
            error_message: None,
        }
    }

    /// Create a failed validation result.
    pub fn failure(code: i32, message: impl Into<String>) -> Self {
        Self {
            valid: false,
            error_code: code,
            error_message: Some(message.into()),
        }
    }
}

/// Extracted token payload.
#[derive(Debug, Clone)]
pub struct TokenPayload {
    /// Token subject (user ID).
    pub subject: String,
    /// Space-separated scopes.
    pub scopes: String,
    /// Token audience.
    pub audience: String,
    /// Expiration timestamp (unix seconds).
    pub expiration: u64,
}

// FFI function type definitions
type GopherAuthInitFn = unsafe extern "C" fn() -> c_int;
type GopherAuthClientCreateFn = unsafe extern "C" fn(
    out: *mut *mut c_void,
    jwks_uri: *const c_char,
    issuer: *const c_char,
) -> c_int;
type GopherAuthClientDestroyFn = unsafe extern "C" fn(client: *mut c_void);
type GopherAuthSetOptionFn =
    unsafe extern "C" fn(client: *mut c_void, key: *const c_char, value: *const c_char) -> c_int;
type GopherAuthValidateTokenFn = unsafe extern "C" fn(
    client: *mut c_void,
    token: *const c_char,
    clock_skew: c_uint,
    out_valid: *mut c_int,
    out_error: *mut *mut c_char,
) -> c_int;
type GopherAuthExtractPayloadFn =
    unsafe extern "C" fn(client: *mut c_void, token: *const c_char, out: *mut *mut c_void) -> c_int;
type GopherAuthPayloadGetSubjectFn = unsafe extern "C" fn(payload: *mut c_void) -> *const c_char;
type GopherAuthPayloadGetScopesFn = unsafe extern "C" fn(payload: *mut c_void) -> *const c_char;
type GopherAuthPayloadGetAudienceFn = unsafe extern "C" fn(payload: *mut c_void) -> *const c_char;
type GopherAuthPayloadGetExpirationFn = unsafe extern "C" fn(payload: *mut c_void) -> u64;
type GopherAuthPayloadDestroyFn = unsafe extern "C" fn(payload: *mut c_void);
type GopherAuthFreeStringFn = unsafe extern "C" fn(s: *mut c_char);

/// Client for gopher-auth native library.
///
/// Provides JWT validation and payload extraction using the gopher-auth C library.
///
/// # Thread Safety
///
/// The client is `Send` and `Sync`, allowing it to be shared across threads.
pub struct GopherAuthClient {
    /// Opaque handle to the native client.
    handle: *mut c_void,
    /// Reference to the loaded library.
    library: Arc<Library>,
}

// Safety: The native library handles are thread-safe when used correctly
unsafe impl Send for GopherAuthClient {}
unsafe impl Sync for GopherAuthClient {}

impl GopherAuthClient {
    /// Create a new gopher-auth client.
    ///
    /// # Arguments
    ///
    /// * `jwks_uri` - URI to fetch JWKS from
    /// * `issuer` - Expected token issuer
    ///
    /// # Returns
    ///
    /// A new client instance or an error if initialization failed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = GopherAuthClient::new(
    ///     "https://auth.example.com/.well-known/jwks.json",
    ///     "https://auth.example.com"
    /// )?;
    /// ```
    pub fn new(jwks_uri: &str, issuer: &str) -> Result<Self, Error> {
        // Load the native library
        let library = Self::load_library()?;
        let library = Arc::new(library);

        // Initialize the library
        unsafe {
            let init: Symbol<GopherAuthInitFn> = library
                .get(b"gopher_auth_init\0")
                .map_err(|e| Error::auth(format!("Failed to load gopher_auth_init: {}", e)))?;

            let result = init();
            if result != 0 {
                return Err(Error::auth(format!(
                    "gopher_auth_init failed with code {}",
                    result
                )));
            }
        }

        // Create the client
        let jwks_uri_c =
            CString::new(jwks_uri).map_err(|e| Error::auth(format!("Invalid jwks_uri: {}", e)))?;
        let issuer_c =
            CString::new(issuer).map_err(|e| Error::auth(format!("Invalid issuer: {}", e)))?;

        let handle = unsafe {
            let create: Symbol<GopherAuthClientCreateFn> =
                library.get(b"gopher_auth_client_create\0").map_err(|e| {
                    Error::auth(format!("Failed to load gopher_auth_client_create: {}", e))
                })?;

            let mut handle: *mut c_void = ptr::null_mut();
            let result = create(&mut handle, jwks_uri_c.as_ptr(), issuer_c.as_ptr());

            if result != 0 || handle.is_null() {
                return Err(Error::auth(format!(
                    "gopher_auth_client_create failed with code {}",
                    result
                )));
            }

            handle
        };

        Ok(Self { handle, library })
    }

    /// Load the native library from known locations.
    fn load_library() -> Result<Library, Error> {
        // Try multiple library names - the auth functions are in libgopher-orch
        let lib_names = if cfg!(target_os = "macos") {
            vec![
                "libgopher-orch.dylib",
                "libgopher-orch.0.dylib",
                "libgopher_orch.dylib",
            ]
        } else if cfg!(target_os = "windows") {
            vec!["gopher-orch.dll", "libgopher-orch.dll", "gopher_orch.dll"]
        } else {
            vec![
                "libgopher-orch.so",
                "libgopher-orch.so.0",
                "libgopher_orch.so",
            ]
        };

        // Build search paths including environment-specified locations
        let mut search_paths = vec![
            String::new(), // Current directory / system paths
            String::from("./"),
            String::from("./native/lib/"),
            String::from("../native/lib/"),
        ];

        // Add paths from DYLD_LIBRARY_PATH / LD_LIBRARY_PATH
        if let Ok(lib_path) = std::env::var("DYLD_LIBRARY_PATH") {
            for path in lib_path.split(':') {
                if !path.is_empty() {
                    let mut p = path.to_string();
                    if !p.ends_with('/') {
                        p.push('/');
                    }
                    search_paths.push(p);
                }
            }
        }
        if let Ok(lib_path) = std::env::var("LD_LIBRARY_PATH") {
            for path in lib_path.split(':') {
                if !path.is_empty() {
                    let mut p = path.to_string();
                    if !p.ends_with('/') {
                        p.push('/');
                    }
                    search_paths.push(p);
                }
            }
        }

        // Add standard system paths
        search_paths.push(String::from("/usr/local/lib/"));
        search_paths.push(String::from("/usr/lib/"));

        for path in &search_paths {
            for name in &lib_names {
                let full_path = format!("{}{}", path, name);
                if let Ok(lib) = unsafe { Library::new(&full_path) } {
                    return Ok(lib);
                }
            }
        }

        Err(Error::auth(format!(
            "Failed to load gopher-auth library. Tried: {:?}",
            lib_names
        )))
    }

    /// Check if the gopher-auth library is available.
    ///
    /// # Returns
    ///
    /// `true` if the library can be loaded, `false` otherwise.
    pub fn is_available() -> bool {
        Self::load_library().is_ok()
    }

    /// Validate a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    /// * `clock_skew` - Allowed clock skew in seconds
    ///
    /// # Returns
    ///
    /// Validation result indicating success or failure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = client.validate_token("eyJ...", 60);
    /// if result.valid {
    ///     println!("Token is valid!");
    /// } else {
    ///     println!("Validation failed: {:?}", result.error_message);
    /// }
    /// ```
    pub fn validate_token(&self, token: &str, clock_skew: u32) -> ValidationResult {
        let token_c = match CString::new(token) {
            Ok(s) => s,
            Err(e) => return ValidationResult::failure(-1, format!("Invalid token string: {}", e)),
        };

        unsafe {
            let validate: Symbol<GopherAuthValidateTokenFn> =
                match self.library.get(b"gopher_auth_validate_token\0") {
                    Ok(f) => f,
                    Err(e) => {
                        return ValidationResult::failure(
                            -1,
                            format!("Failed to load validate function: {}", e),
                        )
                    }
                };

            let mut valid: c_int = 0;
            let mut error: *mut c_char = ptr::null_mut();

            let result = validate(
                self.handle,
                token_c.as_ptr(),
                clock_skew,
                &mut valid,
                &mut error,
            );

            if result != 0 {
                let error_msg = if !error.is_null() {
                    let msg = CStr::from_ptr(error).to_string_lossy().into_owned();
                    self.free_string(error);
                    msg
                } else {
                    format!("Validation failed with code {}", result)
                };
                return ValidationResult::failure(result, error_msg);
            }

            if valid != 0 {
                ValidationResult::success()
            } else {
                let error_msg = if !error.is_null() {
                    let msg = CStr::from_ptr(error).to_string_lossy().into_owned();
                    self.free_string(error);
                    msg
                } else {
                    "Token validation failed".to_string()
                };
                ValidationResult::failure(-2, error_msg)
            }
        }
    }

    /// Extract payload from a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    ///
    /// # Returns
    ///
    /// Extracted token payload or an error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let payload = client.extract_payload("eyJ...")?;
    /// println!("User: {}", payload.subject);
    /// println!("Scopes: {}", payload.scopes);
    /// ```
    pub fn extract_payload(&self, token: &str) -> Result<TokenPayload, Error> {
        let token_c =
            CString::new(token).map_err(|e| Error::auth(format!("Invalid token string: {}", e)))?;

        unsafe {
            let extract: Symbol<GopherAuthExtractPayloadFn> = self
                .library
                .get(b"gopher_auth_extract_payload\0")
                .map_err(|e| Error::auth(format!("Failed to load extract function: {}", e)))?;

            let mut payload: *mut c_void = ptr::null_mut();
            let result = extract(self.handle, token_c.as_ptr(), &mut payload);

            if result != 0 || payload.is_null() {
                return Err(Error::auth(format!(
                    "Failed to extract payload, code {}",
                    result
                )));
            }

            // Extract fields from payload
            let subject = self.get_payload_string(payload, b"gopher_auth_payload_get_subject\0")?;
            let scopes = self.get_payload_string(payload, b"gopher_auth_payload_get_scopes\0")?;
            let audience =
                self.get_payload_string(payload, b"gopher_auth_payload_get_audience\0")?;
            let expiration = self.get_payload_expiration(payload)?;

            // Destroy the payload
            self.destroy_payload(payload);

            Ok(TokenPayload {
                subject,
                scopes,
                audience,
                expiration,
            })
        }
    }

    /// Get a string field from a payload.
    unsafe fn get_payload_string(
        &self,
        payload: *mut c_void,
        fn_name: &[u8],
    ) -> Result<String, Error> {
        let get_fn: Symbol<GopherAuthPayloadGetSubjectFn> = self
            .library
            .get(fn_name)
            .map_err(|e| Error::auth(format!("Failed to load getter function: {}", e)))?;

        let ptr = get_fn(payload);
        if ptr.is_null() {
            return Ok(String::new());
        }

        Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }

    /// Get the expiration field from a payload.
    unsafe fn get_payload_expiration(&self, payload: *mut c_void) -> Result<u64, Error> {
        let get_fn: Symbol<GopherAuthPayloadGetExpirationFn> = self
            .library
            .get(b"gopher_auth_payload_get_expiration\0")
            .map_err(|e| Error::auth(format!("Failed to load expiration getter: {}", e)))?;

        Ok(get_fn(payload))
    }

    /// Destroy a payload handle.
    unsafe fn destroy_payload(&self, payload: *mut c_void) {
        if let Ok(destroy) = self
            .library
            .get::<GopherAuthPayloadDestroyFn>(b"gopher_auth_payload_destroy\0")
        {
            destroy(payload);
        }
    }

    /// Free a string allocated by the native library.
    unsafe fn free_string(&self, s: *mut c_char) {
        if let Ok(free) = self
            .library
            .get::<GopherAuthFreeStringFn>(b"gopher_auth_free_string\0")
        {
            free(s);
        }
    }

    /// Set a client option.
    ///
    /// # Arguments
    ///
    /// * `key` - Option key
    /// * `value` - Option value
    ///
    /// # Returns
    ///
    /// Ok if successful, Err otherwise.
    ///
    /// # Common Options
    ///
    /// - `cache_duration` - JWKS cache duration in seconds
    /// - `auto_refresh` - Enable automatic JWKS refresh ("true"/"false")
    /// - `request_timeout` - HTTP request timeout in seconds
    pub fn set_option(&self, key: &str, value: &str) -> Result<(), Error> {
        let key_c =
            CString::new(key).map_err(|e| Error::auth(format!("Invalid option key: {}", e)))?;
        let value_c =
            CString::new(value).map_err(|e| Error::auth(format!("Invalid option value: {}", e)))?;

        unsafe {
            let set_option: Symbol<GopherAuthSetOptionFn> = self
                .library
                .get(b"gopher_auth_client_set_option\0")
                .map_err(|e| Error::auth(format!("Failed to load set_option: {}", e)))?;

            let result = set_option(self.handle, key_c.as_ptr(), value_c.as_ptr());
            if result != 0 {
                return Err(Error::auth(format!(
                    "Failed to set option '{}', code {}",
                    key, result
                )));
            }
        }

        Ok(())
    }

    /// Explicitly destroy the client handle.
    ///
    /// This is called automatically by Drop, but can be called manually
    /// to release resources early.
    pub fn destroy(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                if let Ok(destroy) = self
                    .library
                    .get::<GopherAuthClientDestroyFn>(b"gopher_auth_client_destroy\0")
                {
                    destroy(self.handle);
                }
            }
            self.handle = ptr::null_mut();
        }
    }

    /// Create a dummy client for testing purposes.
    ///
    /// This client has a null handle and no library, and should only be
    /// used in tests that need to check if a client exists without actually
    /// performing any operations.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self {
            handle: ptr::null_mut(),
            library: Arc::new(unsafe {
                // Create a dummy library reference that won't be used
                // This is safe because we never call any functions on it
                Library::new("/dev/null").unwrap_or_else(|_| {
                    // If /dev/null doesn't work, try a path that definitely exists
                    #[cfg(target_os = "macos")]
                    {
                        Library::new("/usr/lib/libSystem.B.dylib")
                            .expect("Failed to load system library for test dummy")
                    }
                    #[cfg(target_os = "linux")]
                    {
                        Library::new("/lib/x86_64-linux-gnu/libc.so.6")
                            .or_else(|_| Library::new("/lib/libc.so.6"))
                            .expect("Failed to load system library for test dummy")
                    }
                    #[cfg(target_os = "windows")]
                    {
                        Library::new("kernel32.dll")
                            .expect("Failed to load system library for test dummy")
                    }
                })
            }),
        }
    }
}

impl Drop for GopherAuthClient {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert_eq!(result.error_code, 0);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_validation_result_failure() {
        let result = ValidationResult::failure(-1, "Token expired");
        assert!(!result.valid);
        assert_eq!(result.error_code, -1);
        assert_eq!(result.error_message, Some("Token expired".to_string()));
    }

    #[test]
    fn test_token_payload_fields() {
        let payload = TokenPayload {
            subject: "user123".to_string(),
            scopes: "openid profile".to_string(),
            audience: "my-app".to_string(),
            expiration: 1234567890,
        };

        assert_eq!(payload.subject, "user123");
        assert_eq!(payload.scopes, "openid profile");
        assert_eq!(payload.audience, "my-app");
        assert_eq!(payload.expiration, 1234567890);
    }

    #[test]
    fn test_token_payload_clone() {
        let payload = TokenPayload {
            subject: "user".to_string(),
            scopes: "read write".to_string(),
            audience: "api".to_string(),
            expiration: 9999999999,
        };

        let cloned = payload.clone();
        assert_eq!(payload.subject, cloned.subject);
        assert_eq!(payload.scopes, cloned.scopes);
    }

    #[test]
    fn test_is_available() {
        // Just check it doesn't panic - result depends on library presence
        let _ = GopherAuthClient::is_available();
    }

    // Note: The following tests require the native library to be installed
    // They are marked as ignored by default and can be run with:
    // cargo test --features auth --ignored

    #[test]
    #[ignore]
    fn test_client_creation() {
        let result = GopherAuthClient::new(
            "https://example.com/.well-known/jwks.json",
            "https://example.com",
        );

        // This may fail if the library is not installed
        // That's expected in CI environments without the native library
        if let Err(e) = result {
            println!(
                "Client creation failed (expected without native lib): {}",
                e
            );
        }
    }

    #[test]
    #[ignore]
    fn test_client_validate_token() {
        let client = match GopherAuthClient::new(
            "https://example.com/.well-known/jwks.json",
            "https://example.com",
        ) {
            Ok(c) => c,
            Err(_) => return, // Skip if library not available
        };

        // This will fail because the token is invalid
        let result = client.validate_token("invalid.token.here", 0);
        assert!(!result.valid);
    }
}
