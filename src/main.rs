use std::fs::{File};
use std::io::{Write};
use walkdir::WalkDir;
use std::io::{BufRead, BufReader, BufWriter};

const EDITABLE_SINGLE_TAGS: &[&str] = &[
    "br", "hr", "img", "input", "link", "meta", "source", "track", "wbr"
];
const EDITABLE_TAGS: &[&str] = &[
    "blockquote", "figcaption", "progress", "textarea", "address", "article", "caption", "details",
    "section", "summary", "button", "canvas", "figure", "footer", "header", "iframe", "legend",
    "output", "strong", "aside", "label", "meter", "small", "table", "tbody", "tfoot", "thead",
    "abbr", "cite", "code", "data", "form", "mark", "ruby", "samp", "span", "time", "bdi", "bdo",
    "del", "dfn", "div", "ins", "kbd", "nav", "pre", "sub", "sup", "var", "wbr", "img", "dd", "dl",
    "dt", "em", "h1", "h2", "h3", "h4", "h5", "h6", "li", "rp", "rt", "td", "th", "tr", "a", "b",
    "i", "p", "q", "s", "u"
];

fn check(line: &str) -> (bool, bool) {
    for tag in EDITABLE_TAGS.iter() {
        if line.contains(&format!("<{tag}")) && line.find("editable").is_some() {
            let is_single_tag = EDITABLE_SINGLE_TAGS.contains(tag);
            return (true, is_single_tag);
        }
    }
    (false, false)
}

fn add_editable(line: &str, name: &str, website_id: &str, current_id: &mut i32) -> String {
    let (is_editable, is_single_tag) = check(line);
    if is_editable {
        let mut pos: usize = 0;
        let mut found: bool = false;
        for (index, char) in line.chars().enumerate() {
            if char == '<' && found {
                break;
            }
            else if(is_single_tag && char == '/'){
                pos = index;
                break;
            }
            else if char == '>' {
                found = true;
                pos = index;
            }
        }

        let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);

        *current_id += 1;
        let (before, after) = line.split_at(pos);
        return format!("{} {}{}", before, new_id, after);

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
    let root_dir = "C:/Users/erlan/RustroverProjects/Bundler/Simple";

    let name = "exampleName"; // Example name, replace with actual value
    let website_id = "exampleWebsiteId"; // Example website ID, replace with actual value
    let mut current_id = 0;

    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() && entry.file_name() == "node_modules" {
            continue;
        }
        if entry.file_type().is_file() && entry.file_name() == "CatalogButton.tsx" {
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