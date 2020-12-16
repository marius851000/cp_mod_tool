use std::fs::File;
use std::io;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use crate::ModConfiguration;

use walkdir::WalkDir;
use zip::{
    write::{FileOptions, ZipWriter},
    CompressionMethod,
};

const CONFIG_RELATIVE_PATH_TOML: &str = "config.toml";
const CONFIG_RELATIVE_PATH_JSON: &str = "config.json";

#[derive(thiserror::Error, Debug)]
pub enum ModFileWriterError {
    #[error("io error while reading {0}")]
    FileIOError(PathBuf, #[source] io::Error),
    #[error("error while parsing the toml file {0}")]
    TomlDecodeError(PathBuf, #[source] toml::de::Error),
    #[error("error while running walkdir")]
    WalkDirError(#[from] walkdir::Error),
}

pub struct ModFileWriter {
    source_dir: PathBuf,
}

impl ModFileWriter {
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }

    //TODO: better logging (use a progress bar library ? color ?)
    pub fn write<D: Write + Seek>(&self, destination: &mut D) -> Result<(), ModFileWriterError> {
        println!("reading the configuration file");
        let source_config_file_path = self.source_dir.join(CONFIG_RELATIVE_PATH_TOML);
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

        println!("creating the zip file");
        let mut zip = ZipWriter::new(destination);
        zip.set_comment(format!(
            "mod id : {}\nmod name : {}",
            &config.identifier, &config.display_name
        ));

        let walkdir = WalkDir::new(&self.source_dir).follow_links(true);

        let zip_options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        let mut actual_zip_content = Vec::new();
        for entry in walkdir {
            let entry = entry?;

            let content_abs_path = entry.path();
            let content_rel_path = content_abs_path.strip_prefix(&self.source_dir).unwrap(); //TODO:

            if entry.file_type().is_file() {
                if content_rel_path == PathBuf::from(CONFIG_RELATIVE_PATH_TOML) {
                    continue;
                };
                println!("adding the file {:?} to the archive", content_rel_path);
                zip.start_file(content_rel_path.to_string_lossy(), zip_options)
                    .unwrap(); //TODO:
                let mut actual_zip_file = File::open(content_abs_path).unwrap(); //TODO:
                actual_zip_file
                    .read_to_end(&mut actual_zip_content)
                    .unwrap(); //TODO
                zip.write_all(&actual_zip_content).unwrap(); //TODO:
                actual_zip_content.clear();
            } else {
                println!("adding the directory {:?} to the archive", content_rel_path);
                zip.add_directory(content_rel_path.to_string_lossy(), zip_options)
                    .unwrap(); //TODO:
            }
        }

        println!("adding the {} file", CONFIG_RELATIVE_PATH_JSON);
        let config_toml = serde_json::to_vec_pretty(&config).unwrap(); //TODO:
        zip.start_file(CONFIG_RELATIVE_PATH_JSON, zip_options)
            .unwrap(); //TODO:
        zip.write_all(&config_toml).unwrap(); //TODO:
        println!("finished");

        Ok(())
    }
}
