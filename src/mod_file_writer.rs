use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Seek, Write};
use std::path::PathBuf;

use crate::style::StepProgress;
use crate::ModConfiguration;

use walkdir::WalkDir;
use zip::{
    write::{FileOptions, ZipWriter},
    CompressionMethod,
};

use ignore;
use ignore::gitignore::GitignoreBuilder;

const CONFIG_RELATIVE_PATH_TOML: &str = "config.toml";
const CONFIG_RELATIVE_PATH_JSON: &str = "config.json";
const IGNORE_DEFAULT: &str = ".modignore";

#[derive(thiserror::Error, Debug)]
pub enum ModFileWriterError {
    #[error("io error while reading {0}")]
    FileIOError(PathBuf, #[source] io::Error),
    #[error("error while parsing the toml file {0}")]
    TomlDecodeError(PathBuf, #[source] toml::de::Error),
    #[error("error while running walkdir")]
    WalkDirError(#[from] walkdir::Error),
    #[error("error stripping a the path {0} with {1}")]
    StripPrefixError(PathBuf, PathBuf, #[source] std::path::StripPrefixError),
    #[error("error while handling the zip file")]
    ZipError(#[from] zip::result::ZipError),
    #[error("error while writing to the zip file")]
    ZipWriteError(#[source] io::Error),
    #[error("can't generate the output json configuration file, but can parse the toml one. Probably internal error")]
    EncodeJsonError(#[source] serde_json::error::Error),
    #[error("error while handling the ignore file")] // this one should include path
    IgnoreFileError(#[from] ignore::Error),
}

pub struct ModFileWriter {
    source_dir: PathBuf,
}

impl ModFileWriter {
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }

    pub fn write<D: Write + Seek>(&self, destination: &mut D) -> Result<(), ModFileWriterError> {
        let mut progress = StepProgress::new(5);
        let source_config_file_path = self.source_dir.join(CONFIG_RELATIVE_PATH_TOML);
        progress.progress(&format!(
            "reading the {:?} configuration file",
            source_config_file_path
        ));
        let mut source_config_file = File::open(&source_config_file_path)
            .map_err(|err| ModFileWriterError::FileIOError(source_config_file_path.clone(), err))?;
        let mut source_config_content = Vec::new();
        source_config_file
            .read_to_end(&mut source_config_content)
            .map_err(|err| ModFileWriterError::FileIOError(source_config_file_path.clone(), err))?;
        let config =
            toml::from_slice::<ModConfiguration>(&source_config_content).map_err(|err| {
                ModFileWriterError::TomlDecodeError(source_config_file_path.clone(), err)
            })?;

        progress.progress("reading the ignore list");
        let ignore_path = self.source_dir.join(IGNORE_DEFAULT);

        let mut builder = GitignoreBuilder::new(&self.source_dir);
        let ignore = match builder.add(&ignore_path) {
            None => Some(builder.build()?),
            Some(err) => match err.io_error() {
                Some(io_err) => match io_err.kind() {
                    ErrorKind::NotFound => {
                        println!("{:?} not found, ignoring it.", ignore_path);
                        None
                    }
                    _ => return Err(ModFileWriterError::from(err)),
                },
                None => return Err(ModFileWriterError::from(err)),
            },
        };

        progress.progress("creating the zip file");

        let mut zip = ZipWriter::new(destination);
        zip.set_comment(format!(
            "mod id : {}\nmod name : {}",
            &config.identifier, &config.display_name
        ));

        let walkdir = WalkDir::new(&self.source_dir).follow_links(true);

        let zip_options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        let mut embedded_content = Vec::new();
        for entry in walkdir {
            let entry = entry?;

            let content_abs_path = entry.path();
            let content_rel_path =
                content_abs_path
                    .strip_prefix(&self.source_dir)
                    .map_err(|err| {
                        ModFileWriterError::StripPrefixError(
                            content_abs_path.to_path_buf(),
                            self.source_dir.to_path_buf(),
                            err,
                        )
                    })?;

            let is_file = entry.file_type().is_file();

            if let Some(ignore) = &ignore {
                if ignore
                    .matched_path_or_any_parents(&content_rel_path, !is_file)
                    .is_ignore()
                {
                    println!("ignored {:?}", content_rel_path);
                    continue;
                }
            };

            if is_file {
                if content_rel_path == PathBuf::from(CONFIG_RELATIVE_PATH_TOML) {
                    continue;
                };
                println!("adding the file {:?} to the archive", content_rel_path);
                zip.start_file(content_rel_path.to_string_lossy(), zip_options)?;
                let mut embedded_file = File::open(content_abs_path).map_err(|err| {
                    ModFileWriterError::FileIOError(content_abs_path.to_path_buf(), err)
                })?;
                embedded_file
                    .read_to_end(&mut embedded_content)
                    .map_err(|err| {
                        ModFileWriterError::FileIOError(content_abs_path.to_path_buf(), err)
                    })?;
                zip.write_all(&embedded_content)
                    .map_err(ModFileWriterError::ZipWriteError)?;
                embedded_content.clear();
            } else {
                println!("adding the directory {:?} to the archive", content_rel_path);
                zip.add_directory(content_rel_path.to_string_lossy(), zip_options)?;
            }
        }

        progress.progress(&format!(
            "adding the {} configuration file",
            CONFIG_RELATIVE_PATH_JSON
        ));
        let config_toml =
            serde_json::to_vec_pretty(&config).map_err(ModFileWriterError::EncodeJsonError)?;
        zip.start_file(CONFIG_RELATIVE_PATH_JSON, zip_options)?;
        zip.write_all(&config_toml)
            .map_err(ModFileWriterError::ZipWriteError)?;

        progress.progress("finished");
        Ok(())
    }
}
