use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;

// Type alias for Scheme pointers
type Ptr = *const c_void;

#[link(name = "kernel", kind = "static")]
#[link(name = "lz4", kind = "static")]
#[link(name = "z", kind = "static")]
#[link(name = "ncurses", kind = "dylib")]
extern "C" {
    fn Sinteger(n: isize) -> Ptr;
    fn Sflonum(n: f64) -> Ptr;
    fn Sstring(s: *const c_char) -> Ptr;

    fn Sscheme_init(argc: c_int) -> c_int;
    fn Sregister_boot_file(s: *const c_char);
    fn Sbuild_heap(s: *const c_char, f: extern "C" fn());
    fn Sforeign_symbol(name: *const c_char, ptr: *const ());
    fn Sscheme_deinit();
    fn Stop_level_value(s: Ptr) -> Ptr;
    fn Scall0(code: Ptr) -> Ptr;
    fn Scall1(code: Ptr, arg: Ptr) -> Ptr;
    fn Sstring_to_symbol(s: *const c_char) -> Ptr;
}

const SVOID: Ptr = 0x2E as Ptr;

#[inline]
fn seof_objectp(x: Ptr) -> bool {
    x as usize == 0x36
}

extern "C" fn custom_init() {
    // This function intentionally left empty
}

#[no_mangle]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    println!("Hello from rust!");
    a + b
}

// Define a trait for types that can be converted to Chez Scheme values
pub trait ChezValue {
    fn to_chez(&self) -> Ptr;
}

// Implement ChezValue for integers (i32)
impl ChezValue for i32 {
    fn to_chez(&self) -> Ptr {
        unsafe { Sinteger(*self as isize) }
    }
}

// Implement ChezValue for integers (i64)
impl ChezValue for i64 {
    fn to_chez(&self) -> Ptr {
        unsafe { Sinteger(*self as isize) }
    }
}

// Implement ChezValue for floats (f64)
impl ChezValue for f64 {
    fn to_chez(&self) -> Ptr {
        unsafe { Sflonum(*self) }
    }
}

// Implement ChezValue for strings (&str)
impl ChezValue for &str {
    fn to_chez(&self) -> Ptr {
        let c_string = CString::new(*self).unwrap();
        unsafe { Sstring(c_string.as_ptr()) }
    }
}

// Implement ChezValue for String
impl ChezValue for String {
    fn to_chez(&self) -> Ptr {
        let c_string = CString::new(self.as_str()).unwrap();
        unsafe { Sstring(c_string.as_ptr()) }
    }
}

pub trait ChezSymbol {
    fn chez_symbol(&self) -> Ptr;
    fn chez_resolve(&self) -> Ptr;
}

fn resolve_symbol(symbol: Ptr) -> Ptr {
    unsafe { Stop_level_value(symbol) }
}

// Implement ChezSymbol for &str
impl ChezSymbol for &str {
    fn chez_symbol(&self) -> Ptr {
        let c_string = CString::new(*self).unwrap();
        unsafe { Sstring_to_symbol(c_string.as_ptr()) }
    }

    fn chez_resolve(&self) -> Ptr {
        let symbol = self.chez_symbol();
        resolve_symbol(symbol)
    }
}

// Implement ChezSymbol for String
impl ChezSymbol for String {
    fn chez_symbol(&self) -> Ptr {
        let c_string = CString::new(self.as_str()).unwrap();
        unsafe { Sstring_to_symbol(c_string.as_ptr()) }
    }

    fn chez_resolve(&self) -> Ptr {
        let symbol = self.chez_symbol();
        resolve_symbol(symbol)
    }
}

unsafe fn call0(who: &str) -> Ptr {
    Scall0(who.chez_resolve())
}

unsafe fn call1(who: &str, arg: Ptr) -> Ptr {
    Scall1(who.chez_resolve(), arg)
}

unsafe fn eval(code: &str) -> Ptr {
    let input_port = call1("open-input-string", code.to_chez());
    let read_result = call1("read", input_port);
    call1("eval", read_result)
}

fn main() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let petite_boot_file = crate_root.join("scheme/ta6le/petite.boot");
    let scheme_boot_file = crate_root.join("scheme/ta6le/scheme.boot");

    println!("Using petite boot file: {:?}", petite_boot_file);
    println!("Using scheme boot file: {:?}", scheme_boot_file);

    unsafe {
        // Initialize Chez Scheme
        Sscheme_init(0);

        // Register the boot files
        let petite_cstr = &CString::new(petite_boot_file.to_str().unwrap()).unwrap();
        Sregister_boot_file(petite_cstr.as_ptr());

        let scheme_cstr = CString::new(scheme_boot_file.to_str().unwrap()).unwrap();
        Sregister_boot_file(scheme_cstr.as_ptr());

        // Build the heap
        Sbuild_heap(std::ptr::null(), custom_init);

        let name = CString::new("add_numbers").unwrap();
        Sforeign_symbol(name.as_ptr(), add_numbers as *const ());

        eval("(define add-numbers (foreign-procedure \"add_numbers\" (int int) int))");

        let prompt_cstr = CString::new("* ").unwrap();
        // Start the REPL
        loop {
            // Display prompt
            call1("display", Sstring(prompt_cstr.as_ptr()));
            // Read input
            let input = call0("read");

            // Check for EOF
            if seof_objectp(input) {
                break;
            }

            // Evaluate input
            let result = call1("eval", input);

            // Print result if it's not void
            if result != SVOID {
                call1("pretty-print", result);
            }
        }

        // Print newline at end
        call0("newline");

        // Deinitialize Scheme
        Sscheme_deinit();
    }
}
