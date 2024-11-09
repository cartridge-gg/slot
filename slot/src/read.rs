use base64::{engine, Engine};

pub fn read_and_encode_file_as_base64(file_path: Option<String>) -> anyhow::Result<Option<String>> {
    if let Some(path) = file_path {
        let file_contents = std::fs::read(path)?;
        Ok(Some(
            engine::general_purpose::STANDARD.encode(file_contents),
        ))
    } else {
        Ok(None)
    }
}
