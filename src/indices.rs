use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

#[cfg(feature = "logging")]
use tracing::info;

use crate::errors::error::ApiError;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Index {
    pub entries: HashMap<u32, String>,
}

// TODO: needs better refactoring

pub static INDEX: OnceLock<RwLock<Index>> = OnceLock::new();

/// Loads the index from disk and initializes it.
pub async fn initialize_index() -> Result<(), ApiError> {
    #[cfg(feature = "logging")]
    info!("initialize indices");
    let project_dir = std::env::current_dir()?;
    let servers_dir = project_dir.join("servers");
    let index_file = servers_dir.join("index.bin");

    tokio::fs::create_dir_all(&servers_dir).await?;

    if !index_file.exists() {
        let index = Index::default();
        let data = match rmp_serde::to_vec(&index) {
            Ok(data) => data,
            Err(_) => return Err(ApiError::InternalServerError),
        };
        std::fs::write(&index_file, data)?;

        INDEX
            .set(std::sync::RwLock::new(index))
            .map_err(|_| ApiError::InternalServerError)?;
        return Ok(());
    }

    let data = std::fs::read(&index_file)?;
    let index: Index = match rmp_serde::from_slice(&data) {
        Ok(index) => index,
        Err(_) => return Err(ApiError::InternalServerError),
    };

    INDEX.get_or_init(|| std::sync::RwLock::new(index));

    Ok(())
}

/// Adds an entry to the index struct
pub fn add_entry(index: u32, name: String) -> Result<(), ApiError> {
    let mut index_lock = INDEX
        .get()
        .ok_or(ApiError::InternalServerError)?
        .write()
        .map_err(|_| ApiError::InternalServerError)?;
    index_lock.entries.insert(index, name);

    Ok(())
}

/// write the index map to file
pub fn write_index_to_file(path: &std::path::Path) -> Result<(), ApiError> {
    let index_lock = INDEX
        .get()
        .ok_or(ApiError::InternalServerError)?
        .read()
        .map_err(|_| ApiError::InternalServerError)?;
    let serialized = match rmp_serde::to_vec(&*index_lock) {
        Ok(data) => data,
        Err(_) => return Err(ApiError::InternalServerError),
    };
    std::fs::write(path, serialized)?;
    Ok(())
}

/// Get a name by a given index out of the index struct
pub fn get_name_by_index(index: u32) -> Option<String> {
    let index_lock = INDEX.get().unwrap().read().unwrap();
    let name = index_lock.entries.get(&index).cloned();
    drop(index_lock);

    name
}
