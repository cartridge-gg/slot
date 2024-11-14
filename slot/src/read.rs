use base64::{engine, Engine};

pub fn read_and_encode_file_as_base64(file_path: Option<String>) -> anyhow::Result<Option<String>> {
    if let Some(path) = file_path {
        let file_contents = std::fs::read(path)?;
        Ok(Some(base64_encode_bytes(&file_contents)))
    } else {
        Ok(None)
    }
}

pub fn base64_encode_bytes(data: &[u8]) -> String {
    engine::general_purpose::STANDARD.encode(data)
}

pub fn base64_encode_string(data: &str) -> String {
    engine::general_purpose::STANDARD.encode(data)
}

pub fn base64_decode_string(data: &str) -> Result<String, base64::DecodeError> {
    engine::general_purpose::STANDARD
        .decode(data)
        .map(|v| String::from_utf8(v).unwrap())
}
