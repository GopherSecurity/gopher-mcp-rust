//! FFI bindings to the native gopher-orch library.

use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Opaque handle to a native agent
pub type AgentHandle = *mut c_void;

/// Error info structure from native library
#[repr(C)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: *const c_char,
    pub details: *const c_char,
    pub file: *const c_char,
    pub line: i32,
}

// Type aliases for FFI function signatures
type AgentCreateByJsonFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> AgentHandle;
type AgentCreateByApiKeyFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> AgentHandle;
type AgentRunFn = unsafe extern "C" fn(AgentHandle, *const c_char, u64) -> *mut c_char;
type AgentReleaseFn = unsafe extern "C" fn(AgentHandle);
type AgentAddRefFn = unsafe extern "C" fn(AgentHandle);
type ApiFetchServersFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type LastErrorFn = unsafe extern "C" fn() -> *mut ErrorInfo;
type ClearErrorFn = unsafe extern "C" fn();
type FreeFn = unsafe extern "C" fn(*mut c_void);

/// Native library wrapper
struct NativeLibrary {
    _library: Library,
    agent_create_by_json: AgentCreateByJsonFn,
    agent_create_by_api_key: AgentCreateByApiKeyFn,
    agent_run: AgentRunFn,
    agent_release: AgentReleaseFn,
    #[allow(dead_code)]
    agent_add_ref: AgentAddRefFn,
    #[allow(dead_code)]
    api_fetch_servers: ApiFetchServersFn,
    last_error: LastErrorFn,
    clear_error: ClearErrorFn,
    free: FreeFn,
}

unsafe impl Send for NativeLibrary {}
unsafe impl Sync for NativeLibrary {}

static NATIVE_LIB: OnceLock<Option<NativeLibrary>> = OnceLock::new();

fn get_library_path() -> PathBuf {
    // Try to find the library in common locations
    let candidates = [
        // Relative to executable
        PathBuf::from("native/lib"),
        // Relative to current directory
        PathBuf::from("./native/lib"),
        // Absolute path from project root
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("native/lib"),
    ];

    for candidate in &candidates {
        let lib_path = if cfg!(target_os = "macos") {
            candidate.join("libgopher-orch.dylib")
        } else if cfg!(target_os = "windows") {
            candidate.join("gopher-orch.dll")
        } else {
            candidate.join("libgopher-orch.so")
        };

        if lib_path.exists() {
            return lib_path;
        }
    }

    // Fall back to system library search
    if cfg!(target_os = "macos") {
        PathBuf::from("libgopher-orch.dylib")
    } else if cfg!(target_os = "windows") {
        PathBuf::from("gopher-orch.dll")
    } else {
        PathBuf::from("libgopher-orch.so")
    }
}

fn load_library() -> Option<NativeLibrary> {
    let lib_path = get_library_path();

    unsafe {
        let library = Library::new(&lib_path).ok()?;

        let agent_create_by_json: Symbol<AgentCreateByJsonFn> =
            library.get(b"gopher_orch_agent_create_by_json").ok()?;
        let agent_create_by_api_key: Symbol<AgentCreateByApiKeyFn> =
            library.get(b"gopher_orch_agent_create_by_api_key").ok()?;
        let agent_run: Symbol<AgentRunFn> =
            library.get(b"gopher_orch_agent_run").ok()?;
        let agent_release: Symbol<AgentReleaseFn> =
            library.get(b"gopher_orch_agent_release").ok()?;
        let agent_add_ref: Symbol<AgentAddRefFn> =
            library.get(b"gopher_orch_agent_add_ref").ok()?;
        let api_fetch_servers: Symbol<ApiFetchServersFn> =
            library.get(b"gopher_orch_api_fetch_servers").ok()?;
        let last_error: Symbol<LastErrorFn> =
            library.get(b"gopher_orch_last_error").ok()?;
        let clear_error: Symbol<ClearErrorFn> =
            library.get(b"gopher_orch_clear_error").ok()?;
        let free: Symbol<FreeFn> =
            library.get(b"gopher_orch_free").ok()?;

        Some(NativeLibrary {
            _library: library,
            agent_create_by_json: *agent_create_by_json,
            agent_create_by_api_key: *agent_create_by_api_key,
            agent_run: *agent_run,
            agent_release: *agent_release,
            agent_add_ref: *agent_add_ref,
            api_fetch_servers: *api_fetch_servers,
            last_error: *last_error,
            clear_error: *clear_error,
            free: *free,
        })
    }
}

fn get_lib() -> Option<&'static NativeLibrary> {
    NATIVE_LIB.get_or_init(load_library).as_ref()
}

/// Check if the native library is available
pub fn is_available() -> bool {
    get_lib().is_some()
}

/// Create an agent with JSON server configuration
pub fn agent_create_by_json(provider: &str, model: &str, server_json: &str) -> AgentHandle {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return std::ptr::null_mut(),
    };

    let c_provider = CString::new(provider).unwrap();
    let c_model = CString::new(model).unwrap();
    let c_server_json = CString::new(server_json).unwrap();

    unsafe {
        (lib.agent_create_by_json)(
            c_provider.as_ptr(),
            c_model.as_ptr(),
            c_server_json.as_ptr(),
        )
    }
}

/// Create an agent with API key
pub fn agent_create_by_api_key(provider: &str, model: &str, api_key: &str) -> AgentHandle {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return std::ptr::null_mut(),
    };

    let c_provider = CString::new(provider).unwrap();
    let c_model = CString::new(model).unwrap();
    let c_api_key = CString::new(api_key).unwrap();

    unsafe {
        (lib.agent_create_by_api_key)(
            c_provider.as_ptr(),
            c_model.as_ptr(),
            c_api_key.as_ptr(),
        )
    }
}

/// Run a query against the agent
pub fn agent_run(agent: AgentHandle, query: &str, timeout_ms: u64) -> String {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return String::new(),
    };

    if agent.is_null() {
        return String::new();
    }

    let c_query = CString::new(query).unwrap();

    unsafe {
        let result = (lib.agent_run)(agent, c_query.as_ptr(), timeout_ms);
        if result.is_null() {
            return String::new();
        }

        let rust_string = CStr::from_ptr(result).to_string_lossy().into_owned();
        (lib.free)(result as *mut c_void);
        rust_string
    }
}

/// Release an agent handle
pub fn agent_release(agent: AgentHandle) {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return,
    };

    if !agent.is_null() {
        unsafe {
            (lib.agent_release)(agent);
        }
    }
}

/// Get the last error message
pub fn get_last_error() -> String {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return String::new(),
    };

    unsafe {
        let error_info = (lib.last_error)();
        if error_info.is_null() || (*error_info).message.is_null() {
            return String::new();
        }
        CStr::from_ptr((*error_info).message).to_string_lossy().into_owned()
    }
}

/// Clear the last error
pub fn clear_error() {
    let lib = match get_lib() {
        Some(lib) => lib,
        None => return,
    };

    unsafe {
        (lib.clear_error)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        // Just check it doesn't panic
        let _ = is_available();
    }
}
