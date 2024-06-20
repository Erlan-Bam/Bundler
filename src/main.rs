use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use walkdir::WalkDir;
fn add_editable(content: String) -> String{
    // Make editable function
}
fn modify_file(file_path: &str) -> std::io::Result<()> {
    let mut file: File = OpenOptions::new().read(true).write(true).open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    content = add_editable(content);

    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let root_dir = "C:/Users/erlan/RustroverProjects/Bundler/Check";
    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                if extension == "jsx" || extension == "tsx" {
                    if let Some(file_path_str) = file_path.to_str() {
                        modify_file(file_path_str)?;
                    }
                }
            }
        }
    }
    Ok(())
}
