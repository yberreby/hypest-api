use rustc_serialize::base64::ToBase64;
use rustc_serialize::base64;
use rustc_serialize::base64::Config;

/// Returns the base64 of a hash
pub fn to_base64(input: &[u8]) -> String {
    /*
        serializes an input into base64
    */
    let config = Config {
        char_set: base64::CharacterSet::Standard,
        newline: base64::Newline::LF,
        pad: false,
        line_length: None
    };

    let hash: String = input.to_base64(config);
    hash
}
