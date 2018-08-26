use config::Config;
use error::*;
use fileutils::{cleanup_paths, create_backup, get_paths, PathList};
use solver::{solve_rename_order, RenameMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Renamer {
    paths: PathList,
    config: Arc<Config>,
}

impl Renamer {
    pub fn new(config: &Arc<Config>) -> Result<Renamer> {
        let input_paths = get_paths(&config);
        Ok(Renamer {
            paths: input_paths,
            config: config.clone(),
        })
    }

    /// Process path batch
    pub fn process(&mut self) -> Result<()> {
        // Remove directories and on existing paths from the list
        cleanup_paths(&mut self.paths, self.config.dirs);

        // Relate original names with their targets
        let rename_map = self.get_rename_map()?;

        // Solve targets ordering to avoid renaming conflicts
        let rename_order = solve_rename_order(&rename_map)?;

        // Execute actual renaming
        for target in &rename_order {
            let source = &rename_map[target];
            self.rename(source, target)?;
        }

        Ok(())
    }

    /// Replace expression match in the given path using stored config.
    fn replace_match(&self, path: &PathBuf) -> PathBuf {
        let expression = &self.config.expression;
        let replacement = &self.config.replacement;

        let file_name = path.file_name().unwrap().to_str().unwrap();
        let parent = path.parent();

        let target_name = expression.replace(file_name, &replacement[..]);
        match parent {
            None => PathBuf::from(target_name.to_string()),
            Some(path) => path.join(Path::new(&target_name.into_owned())),
        }
    }

    /// Get hash map containing all replacements to be done
    fn get_rename_map(&self) -> Result<RenameMap> {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        let mut rename_map = RenameMap::new();
        let mut error_string = String::new();

        for path in &self.paths {
            let target = self.replace_match(&path);
            // Discard paths with no changes
            if target != *path {
                if let Some(old_path) = rename_map.insert(target.clone(), path.clone()) {
                    // Targets cannot be duplicated by any reason
                    error_string.push_str(&colors
                        .error
                        .paint(format!(
                            "\n{0}->{2}\n{1}->{2}\n",
                            old_path.display(),
                            path.display(),
                            target.display()
                        ))
                        .to_string());
                }
            }
        }

        if error_string.is_empty() {
            Ok(rename_map)
        } else {
            Err(Error {
                kind: ErrorKind::SameFilename,
                value: Some(error_string),
            })
        }
    }

    /// Rename path in the filesystem or simply print renaming information. Checks if target
    /// filename exists before renaming.
    fn rename(&self, source: &PathBuf, target: &PathBuf) -> Result<()> {
        let printer = &self.config.printer;
        let colors = &printer.colors;

        if self.config.force {
            // Create a backup before actual renaming
            if self.config.backup {
                match create_backup(source) {
                    Ok(backup) => printer.print(&format!(
                        "{} Backup created - {}",
                        colors.info.paint("Info: "),
                        colors.source.paint(format!(
                            "{} -> {}",
                            source.display(),
                            backup.display()
                        ))
                    )),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            // Rename paths in the filesystem
            if let Err(err) = fs::rename(&source, &target) {
                return Err(Error {
                    kind: ErrorKind::Rename,
                    value: Some(format!("{} -> {}\n{}", source.display(), target.display(), err)),
                });
            } else {
                printer.print(&format!(
                    "{} -> {}",
                    colors.source.paint(source.to_string_lossy().to_string()),
                    colors.target.paint(target.to_string_lossy().to_string())
                ));
            }
        } else {
            // Just print info in dry-run mode
            printer.print(&format!(
                "{} -> {}",
                colors.source.paint(source.to_string_lossy().to_string()),
                colors.target.paint(target.to_string_lossy().to_string())
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate tempfile;
    use super::*;
    use config::RunMode;
    use output::Printer;
    use regex::Regex;
    use std::fs;
    use std::path::Path;
    use std::process;
    use std::sync::Arc;

    #[test]
    fn renamer() {
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        println!("Running test in '{:?}'", tempdir);
        let temp_path = tempdir.path().to_str().unwrap();

        // Generate a mock directory tree and files
        //
        // - temp_path
        //     |
        //     - test_file_1.txt
        //     |
        //     - test_file_2.txt
        //     |
        //     - mock_dir
        //         |
        //         - test_file_1.txt
        //         |
        //         - test_file_2.txt
        //
        let mock_dir = format!("{}/mock_dir", temp_path);
        let mock_files: Vec<String> = vec![
            format!("{}/test_file_1.txt", temp_path),
            format!("{}/test_file_2.txt", temp_path),
            format!("{}/test_file_1.txt", mock_dir),
            format!("{}/test_file_2.txt", mock_dir),
        ];

        // Create directory tree and files in the filesystem
        fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        for file in &mock_files {
            fs::File::create(&file).expect("Error creating mock file...");
        }

        // Create config
        let mock_config = Arc::new(Config {
            expression: Regex::new("test").unwrap(),
            replacement: "passed".to_string(),
            force: true,
            backup: true,
            dirs: false,
            mode: RunMode::FileList(mock_files),
            printer: Printer::colored(),
        });

        // Run renamer
        let mut renamer = match Renamer::new(&mock_config) {
            Ok(renamer) => renamer,
            Err(err) => {
                mock_config.printer.print_error(&err);
                process::exit(1);
            }
        };
        if let Err(err) = renamer.process() {
            mock_config.printer.print_error(&err);
            process::exit(1);
        }

        // Check renamed files
        assert!(Path::new(&format!("{}/passed_file_1.txt", temp_path)).exists());
        assert!(Path::new(&format!("{}/passed_file_2.txt", temp_path)).exists());
        assert!(Path::new(&format!("{}/passed_file_1.txt", mock_dir)).exists());
        assert!(Path::new(&format!("{}/passed_file_2.txt", mock_dir)).exists());

        // Check backup files
        assert!(Path::new(&format!("{}/test_file_1.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", temp_path)).exists());
        assert!(Path::new(&format!("{}/test_file_1.txt.bk", mock_dir)).exists());
        assert!(Path::new(&format!("{}/test_file_2.txt.bk", mock_dir)).exists());
    }
}
