use std::fs::{File};
use std::io::{Write};
use walkdir::WalkDir;
use std::io::{BufRead, BufReader, BufWriter};

const EDITABLE_TAGS: &[&str] = &[
    "a", "abbr", "address", "article", "aside", "b", "bdi", "bdo", "blockquote", "button", "canvas",
    "caption", "cite", "code", "data", "dd", "del", "details", "dfn", "div", "dl", "dt", "em",
    "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "i",
    "iframe", "ins", "kbd", "label", "legend", "li", "mark", "meter", "nav", "output", "p", "pre",
    "progress", "q", "rp", "rt", "ruby", "s", "samp", "section", "small", "span", "strong", "sub",
    "summary", "sup", "table", "tbody", "td", "textarea", "tfoot", "th", "thead", "time", "tr", "u",
    "var", "wbr"
];

fn check(line: &str) -> bool {
    for tag in EDITABLE_TAGS.iter() {
        if line.contains(&format!("<{}", tag)) && line.find("editable").is_some() {
            return true;
        }
    }
    false
}

fn add_editable(line: &str, name: &str, website_id: &str, current_id: &mut i32) -> String {
    if check(line) {
        if let Some(mut pos) = line.rfind('>') {
            pos -= 1;
            let new_id = format!(r#"id="{}_editable_{}_{}""#, name, website_id, current_id);
            println!("Position: {pos}, New ID: {new_id}");
            *current_id += 1;
            let (before, after) = line.split_at(pos + 1);
            return format!("{before} {new_id} {after}");
        }
    }
    line.to_string()
}

fn modify_file(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{}.tmp", file_path);
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);

    for line in reader.lines() {
        let line = line?;
        let modified_line = add_editable(&line, name, website_id, current_id);
        writeln!(writer, "{}", modified_line)?;
    }

    writer.flush()?;
    std::fs::rename(temp_file_path, file_path)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let root_dir = "C:/Users/erlan/RustroverProjects/Bundler/Test";

    let name = "exampleName"; // Example name, replace with actual value
    let website_id = "exampleWebsiteId"; // Example website ID, replace with actual value
    let mut current_id = 0;

    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                if extension == "jsx" || extension == "tsx" {
                    modify_file(file_path.to_str().unwrap(), name, website_id, &mut current_id)?;
                }
            }
        }
    }
    Ok(())
}