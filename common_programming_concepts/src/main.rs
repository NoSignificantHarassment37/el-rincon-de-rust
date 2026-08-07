use std::slice;
use std::str;

fn main() {
    let story: &str = "Once upon a time...";

    let ptr: *const u8 = story.as_ptr();
    let len: usize = story.len();

    // story has nineteen bytes
    assert_eq!(19, len);

    // We can re-build a str out of ptr and len. This is all unsafe because
    // we are responsible for making sure the two components are valid:
    let s: Result<&str, str::Utf8Error> = unsafe {
        // First, we build a &[u8]...
        let slice: &[u8] = slice::from_raw_parts(ptr, len);

        // ... and then convert that slice into a string slice
        str::from_utf8(slice)
    };

    assert_eq!(s, Ok(story));
}
