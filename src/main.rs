use cp_mod_tool::ModFileWriter;
use std::fs::File;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut destination = File::create("test.zip").unwrap();
    ModFileWriter::new(PathBuf::from("./test/test_mod")).write(&mut destination)?;
    Ok(())
}
