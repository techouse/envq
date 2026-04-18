/// Splits bytes into physical lines while keeping each line terminator separate.
pub(super) fn split_physical_lines(text: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while index < text.len() {
        match text[index] {
            b'\r' => {
                if index + 1 < text.len() && text[index + 1] == b'\n' {
                    segments.push((text[start..index].to_vec(), b"\r\n".to_vec()));
                    index += 2;
                } else {
                    segments.push((text[start..index].to_vec(), b"\r".to_vec()));
                    index += 1;
                }
                start = index;
            }
            b'\n' => {
                segments.push((text[start..index].to_vec(), b"\n".to_vec()));
                index += 1;
                start = index;
            }
            _ => index += 1,
        }
    }

    if start < text.len() {
        segments.push((text[start..].to_vec(), Vec::new()));
    }

    segments
}

/// Chooses the newline style used for appended or repaired terminal lines.
pub(super) fn preferred_newline(segments: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    for (_line_text, newline) in segments {
        if !newline.is_empty() {
            return newline.clone();
        }
    }
    platform_newline().to_vec()
}

#[cfg(windows)]
fn platform_newline() -> &'static [u8] {
    b"\r\n"
}

#[cfg(not(windows))]
fn platform_newline() -> &'static [u8] {
    b"\n"
}
