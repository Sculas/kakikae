use std::fs::File;
use std::io::{ErrorKind, Read};
use crate::error::KakikaeError;

pub fn read_stage_data(path: &str) -> Result<Vec<u8>, KakikaeError> {
    let full_path = env!("STAGE_BUILD_DIR").to_string() + path;
    let stage_data_result = File::open(&full_path);
    match stage_data_result {
        Ok(mut f) => {
            let mut buffer = vec![0; f.metadata()?.len() as usize];
            f.read_exact(&mut buffer)?;
            Ok(buffer)
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    Err(KakikaeError::StageDataNotFound(path.into()))
                }
                _ => {
                    Err(KakikaeError::Io(e))
                }
            }
        }
    }
}