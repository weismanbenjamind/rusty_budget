use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FilesystemError {
    Base(String),
    EmptyPath,
    CreateDirFailed(PathBuf),
}

impl std::fmt::Display for FilesystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FilesystemError::Base(err_msg) => {
                write!(f, "{err_msg}")
            }
            FilesystemError::EmptyPath => {
                write!(
                    f,
                    "Found empty string for output file. Must pass a file or path to a file"
                )
            }
            FilesystemError::CreateDirFailed(target_path) => {
                write!(
                    f,
                    "Failed to create directories leading to file {}",
                    target_path.display()
                )
            }
        }
    }
}

impl std::error::Error for FilesystemError {}

impl From<io::Error> for FilesystemError {
    fn from(value: io::Error) -> Self {
        Self::Base(value.to_string())
    }
}

pub fn generate_subdirs_for_output<P: AsRef<Path>>(
    output_file_path: P,
) -> Result<(), FilesystemError> {
    // Shadow the input output_file_path here
    let output_file_path = output_file_path.as_ref();

    if output_file_path.as_os_str().is_empty() {
        return Err(FilesystemError::EmptyPath);
    }

    // None case is parent is root so we don't need to do anything e.g. Path("/").is_none() == true
    // If there is a parent and it doesn't exist create the filesystem
    // Parent cannot equal empty string if it does not exist empty string means we are at the root of a relative path
    if let Some(parent_dir) = output_file_path.parent()
        && !parent_dir.exists()
        && !parent_dir.as_os_str().is_empty()
    {
        return std::fs::create_dir_all(parent_dir)
            .map_err(|_| FilesystemError::CreateDirFailed(output_file_path.to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_generate_subdirs_for_output_empty_string_filesystem() {
        let result = generate_subdirs_for_output("");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("found empty string")
        );
    }

    #[test]
    fn test_generate_subdirs_for_output_parent_dirs_exist_filesystem() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();

        let new_file_path = temp_dir_path.join("test_file.txt");
        std::fs::write(&new_file_path, "").unwrap();

        let num_sub_comps = std::fs::read_dir(temp_dir_path).unwrap().count();
        assert!(num_sub_comps == 1);

        generate_subdirs_for_output(new_file_path.to_str().unwrap()).unwrap();

        let num_sub_comps = std::fs::read_dir(temp_dir_path).unwrap().count();
        assert!(num_sub_comps == 1);
    }

    #[test]
    fn test_generate_subdir_for_output_parent_dir_not_exist_filesystem() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();

        let new_file_path = temp_dir_path.join("subdir").join("test_file.txt");
        let new_file_path_parent = new_file_path.parent().unwrap();

        assert!(!new_file_path_parent.exists());

        generate_subdirs_for_output(new_file_path.to_str().unwrap()).unwrap();

        assert!(new_file_path_parent.exists() && new_file_path_parent.is_dir());
    }

    #[test]
    fn generate_subdirs_for_output_parent_dir_root_filesystem() {
        let target_file_path = "/output_file.txt";
        let original_num_files = std::fs::read_dir("/").unwrap().count();

        generate_subdirs_for_output(target_file_path).unwrap();

        let new_file_count = std::fs::read_dir("/").unwrap().count();

        assert_eq!(original_num_files, new_file_count);
    }

    #[test]
    fn generate_subdirs_for_output_parent_dir_empty_filesystem() {
        let target_file_path = "output_file.txt";
        let original_num_files = std::fs::read_dir(".").unwrap().count();

        generate_subdirs_for_output(target_file_path).unwrap();

        let new_file_count = std::fs::read_dir(".").unwrap().count();
        assert_eq!(original_num_files, new_file_count);
    }
}
