// ![cfg(any(target_os = "macos", target_os = "ios"))]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/libpd_bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_pd() {
        unsafe {
            let pd = libpd_init();
            assert_eq!(pd, 0);
        }
    }
}
