use anyhow::Result;

// including bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// FFI wrapper
pub struct Layout {
    ptr: *mut std::ffi::c_void,
}

impl Layout {
    pub fn new() -> Result<Self> {
        let ptr = unsafe { init_layout() };
        if ptr.is_null() {
            anyhow::bail!("Failed to initialize layout");
        }
        Ok(Self { ptr })
    }

    pub fn run(&self, input: &str) -> Result<String> {
        unsafe {
            let c_input = std::ffi::CString::new(input)?;
            let result_ptr = run_layout(c_input.as_ptr());
            if result_ptr.is_null() {
                anyhow::bail!("Failed to run layout");
            }
            let result = std::ffi::CStr::from_ptr(result_ptr)
                .to_string_lossy()
                .into_owned();
            free_string(result_ptr);
            Ok(result)
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        unsafe {
            destroy_layout(self.ptr);
        }
    }
}
