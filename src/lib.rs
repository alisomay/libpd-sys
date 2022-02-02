#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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
    // TODO: Write tests for all bindings one day.. :)
}
