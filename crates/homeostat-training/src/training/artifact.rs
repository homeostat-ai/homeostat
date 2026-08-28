use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::error::TrainingError;

pub(super) fn prepare_artifact_dir(path: &Path) -> Result<(), TrainingError> {
    if path.exists() {
        if !path.is_dir() {
            return Err(TrainingError::ArtifactPathIsNotDirectory(
                path.to_path_buf(),
            ));
        }
        if fs::read_dir(path)?.next().transpose()?.is_some() {
            return Err(TrainingError::ArtifactDirectoryNotEmpty(path.to_path_buf()));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub(super) fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), TrainingError> {
    let writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}
