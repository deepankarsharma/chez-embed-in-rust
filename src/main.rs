use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;

// Type alias for Scheme pointers
type Ptr = *mut c_void;

#[link(name = "kernel", kind = "static")]
#[link(name = "lz4", kind = "static")]
#[link(name = "z", kind = "static")]
#[link(name = "ncurses", kind = "dylib")]
extern "C" {
    fn Sscheme_init(argc: c_int) -> c_int;
    fn Sregister_boot_file(s: *const c_char);
    fn Sbuild_heap(s: *const c_char, f: extern "C" fn());
    fn Sscheme_deinit();
    fn Stop_level_value(s: *const c_void) -> Ptr;
    fn Scall0(code: Ptr) -> Ptr;
    fn Scall1(code: Ptr, arg: Ptr) -> Ptr;
    fn Sstring_to_symbol(s: *const c_char) -> Ptr;
    fn Sstring(s: *const c_char) -> Ptr;
}

const SVOID: Ptr = 0x2E as Ptr;

#[inline]
fn seof_objectp(x: Ptr) -> bool {
    x as usize == 0x36
}

extern "C" fn custom_init() {
    // This function intentionally left empty
}

unsafe fn call0(who: &str) -> Ptr {
    let cstr = &CString::new(who).unwrap();
    let symbol = Sstring_to_symbol(cstr.as_ptr());
    let code = Stop_level_value(symbol);
    Scall0(code)
}

unsafe fn call1(who: &str, arg: Ptr) -> Ptr {
    let cstr = &CString::new(who).unwrap();
    let symbol = Sstring_to_symbol(cstr.as_ptr());
    let code = Stop_level_value(symbol);
    Scall1(code, arg)
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
