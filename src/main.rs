use argh::FromArgs;
use cp_mod_tool::ModFileWriter;
use std::fs::File;
use std::path::PathBuf;

#[derive(FromArgs)]
/// Top-level command
struct TopLevel {
    #[argh(subcommand)]
    nested: RootSubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum RootSubCommands {
    Package(PackageCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "package")]
/// Package a mod into a redistributable zip file
struct PackageCommand {
    #[argh(option, default = "PathBuf::from(\".\")")]
    /// the source directory that contain the mod source.
    source_dir: PathBuf,
    #[argh(option)]
    /// path to the zip file that will be created.
    output_file: PathBuf,
}

fn main() -> anyhow::Result<(), anyhow::Error> {
    let command: TopLevel = argh::from_env();
    match &command.nested {
        RootSubCommands::Package(package_command) => {
            let mut destination = File::create(&package_command.output_file).unwrap();
            ModFileWriter::new(PathBuf::from(&package_command.source_dir))
                .write(&mut destination)?;
            Ok(())
        }
    }
}
