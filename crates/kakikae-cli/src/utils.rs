use std::fs::File;
use std::io::{ErrorKind, Read};
use crate::error::KakikaeError;

pub fn read_stage_data(path: &str) -> Result<Vec<u8>, KakikaeError> {
    let stage_data_result = File::open(concat!(env!("STAGE_BUILD_DIR"), path));
    match stage_data_result {
        Ok(mut f) => {
            let mut buffer = vec![0; f.metadata()?.len() as usize];
            f.read(&mut buffer)?;
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