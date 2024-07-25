use std::fs::{File};
use std::io::{Write, BufRead, BufReader, BufWriter};
use walkdir::WalkDir;
use duct::cmd;

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

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

fn check(line: &str) -> (bool, bool, &str) {
    if line.contains("id=") {
        return (false, false, "");
    }
    for tag in EDITABLE_TAGS.iter() {
        if line.contains(&format!("<{tag}")) && line.find("editable").is_some() {
            let is_single_tag = EDITABLE_SINGLE_TAGS.contains(tag);
            return (true, is_single_tag, tag);
        }
    }
    (false, false, "")
}

fn add_editable(line: &str, name: &str, website_id: &str, current_id: &mut i32) -> String {
    let (is_editable, is_single_tag, _tag) = check(line);
    if is_editable {
        let mut pos: usize = 0;
        let mut found: bool = false;
        for (index, char) in line.chars().enumerate() {
            if char == '<' && found {
                break;
            } else if is_single_tag && char == '/' {
                pos = index;
                break;
            } else if char == '>' {
                found = true;
                pos = index;
            }
        }

        let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);

        *current_id += 1;
        let (before, after) = line.split_at(pos);
        return format!("{before} {new_id}{after}");
    }
    line.to_string()
}

fn add_import<W: Write>(writer: &mut W, lines: Vec<String>) -> std::io::Result<()> {
    let import_statement = "import { Wrapper } from \"#spark-admin-sdk\";";

    for line in lines {
        let (is_editable, _is_single_tag, _tag) = check(&line);
        if is_editable {
            writeln!(writer, "{}", import_statement)?;
            break;
        }
    }

    Ok(())
}

fn modify_file(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import(&mut writer, lines)?;

    let mut l: usize = 0;
    while l < lines.len() {
        let line = &lines[l];
        let modified_line = add_editable(&line, name, website_id, current_id);
        let (is_editable, is_single_tag, tag) = check(line);

        if is_editable {
            if is_single_tag {
                writeln!(writer, "{}", modified_line)?;
                l += 1;
                continue;
            }
            let start = leading_spaces(&lines[l]);
            writeln!(writer, "{}<Wrapper />", " ".repeat(start))?;
            writeln!(writer, "{}", modified_line)?;
            let mut r = l + 1;
            while r < lines.len() && !lines[r].contains(&format!("</{}>", tag)) {
                writeln!(writer, "{}", lines[r])?;
                r += 1;
            }
            if r < lines.len() {
                writeln!(writer, "{}", lines[r])?;
            }
            l = r + 1;
            writeln!(writer, "{}<Wrapper />", " ".repeat(start))?;
        } else {
            writeln!(writer, "{}", line)?;
            l += 1;
        }
    }

    writer.flush()?;
    std::fs::rename(temp_file_path, file_path)?;

    Ok(())

}

fn run_build(path: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    let result = cmd!(NPM, "run", "build")
        .dir(path)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run();

    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Build succeeded.");
            } else {
                println!("Build failed with the following error:\n{}", String::from_utf8_lossy(&output.stderr));
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Build failed"));
            }
        }
        Err(err) => {
            println!("Failed to execute build command: {:?}", err);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to execute build command: {:?}", err)));
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let root_dir = "C:/Users/erlan/RustroverProjects/Bundler/Simple";

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

    // run_build(root_dir)?;

    Ok(())
}
