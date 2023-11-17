extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn now() -> f64 {
    web_sys
        ::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
}

pub fn generate_random_u32(min: u32, max: u32) -> u32 {
    let mut rand_array: [u8; 4] = [0u8; 4];
    let crypto = web_sys::window().unwrap().crypto().unwrap();

    crypto.get_random_values_with_u8_array(&mut rand_array).unwrap();

    // Convert the random bytes to an i32 value.
    let random_usize = u32::from_be_bytes(rand_array);

    // Return a random i32 value between the specified min and max values.
    (random_usize % (max - min + 1)) + min
}
