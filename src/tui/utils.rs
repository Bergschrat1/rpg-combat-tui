use color_eyre::eyre::{eyre, Context, Result};
use log::info;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::cli::Args;

/// Returns the path to the save state for the given combat file if it exists
pub fn check_for_save_file(combat_file: &Path) -> Option<PathBuf> {
    let save_extension = combat_file.extension()?.to_str()?.to_string() + ".bkp";
    let save_filename = ".".to_string() + combat_file.file_name().unwrap().to_str().unwrap_or("");
    let save_file = combat_file
        .with_file_name(save_filename)
        .with_extension(save_extension);
    if save_file.exists() {
        Some(save_file)
    } else {
        None
    }
}

/// checks if the given file is a valid yaml file
pub fn validate_yaml_extension(file: &Path) -> Result<bool> {
    match file.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => Ok(ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml")),
        None => Err(eyre!(
            "The file '{}' must have a .yaml or .yml extension.",
            file.display()
        )),
    }
}

pub fn load_combat_yaml(args: &Args) -> Result<String> {
    if let Some(save_file) = check_for_save_file(&args.combat_file) {
        info!("Found savefile: {}", &save_file.display());
        read_file_with_context(&save_file)
    } else {
        info!(
            "Didn't find savefile. Reading from {}.",
            &args.combat_file.display(),
        );
        let mut yaml = read_file_with_context(&args.combat_file)?;

        if let Some(player_path) = &args.player_characters {
            info!("Reading player info from {}", &player_path.display());
            let player_yaml = read_file_with_context(player_path)?;
            yaml = player_yaml + &yaml;
        }
        Ok(yaml)
    }
}

pub fn read_file_with_context(path: &Path) -> Result<String> {
    fs::read_to_string(path).wrap_err_with(|| format!("Failed to read file '{}'.", path.display()))
}
