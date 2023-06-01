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

pub fn is_number(s: &str) -> bool {
    let mut number = true;
    for c in s.chars() {
        if !c.is_ascii_digit() {
            number = false;
            break;
        }
    }
    number
}

const BASE64_ALPHABET: [u8; 64] =
    *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };
        result.push(char::from(BASE64_ALPHABET[(b1 >> 2) as usize]));
        result.push(char::from(
            BASE64_ALPHABET[(((b1 & 3) << 4) | ((b2 >> 4) & 15)) as usize],
        ));
        if i + 1 < data.len() {
            result.push(char::from(
                BASE64_ALPHABET[(((b2 & 15) << 2) | ((b3 >> 6) & 3)) as usize],
            ));
        } else {
            result.push(char::from(b'='));
        }
        if i + 2 < data.len() {
            result.push(char::from(BASE64_ALPHABET[(b3 & 63) as usize]));
        } else {
            result.push(char::from(b'='));
        }
        i += 3;
    }
    result
}


pub fn base64_decode(data: &str) -> Option<Vec<u8>> {
    let data = data.replace("=", "");
    let mut result = Vec::new();
    let mut buffer = 0;
    let mut bits_left = 0;
    for c in data.chars() {
        if let Some(index) = BASE64_ALPHABET.iter().position(|&x| x == c as u8) {
            buffer = (buffer << 6) | index as u32;
            bits_left += 6;
            if bits_left >= 8 {
                bits_left -= 8;
                result.push((buffer >> bits_left) as u8);
            }
        } else {
            return None;
        }
    }
    if bits_left >= 8 {
        return None;
    }

    Some(result)
}