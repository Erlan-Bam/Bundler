use std::fs::{File, rename};
use regex::Regex;
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

fn check(line: & str) -> (bool, & str, & str) {
    if line.contains("id=") {
        return (false, "", "");
    }
    for tag in EDITABLE_TAGS.iter() {
        if line.contains(&format!("<{tag}")) && line.find("editable").is_some() {
            return if tag.to_string() == "img" {
                (true, "image", tag)
            } else if line.contains("editable_section") {
                return (true, "section", tag)
            } else if line.contains("editable_block") {
                return (true, "block", tag)
            } else {
                return (true, "wrapper", tag)
            }
        }
    }
    (false, "", "")
}

fn add_import<W: Write>(writer: &mut W, lines: Vec<String>, operation: &str) -> std::io::Result<()> {
    let mut import_text_wrapper = "import { Wrapper } from \"#spark-admin-sdk\";";
    let mut import_image_popup_wrapper = "import { ImageWrapper } from \"#spark-admin-sdk\";";
    let mut import_section_wrapper = "import { SectionWrapper } from \"#spark-admin-sdk\";";
    let mut import_block_wrapper = "import { BlockWrapper } from \"#spark-admin-sdk\";";

    for line in lines {
        match operation {
            "wrapper" if import_text_wrapper.is_empty() => break,
            "image" if import_image_popup_wrapper.is_empty() => break,
            "block" if import_block_wrapper.is_empty() => break,
            "section" if import_section_wrapper.is_empty() => break,
            _ => (),
        }

        let (is_editable, variant, _tag) = check(&line);

        if is_editable {
            match variant {
                "image" => {
                    if operation == "image" && !import_image_popup_wrapper.is_empty() {
                        writeln!(writer, "{}", import_image_popup_wrapper)?;
                        import_image_popup_wrapper = "";
                    }
                }
                "section" => {
                    if operation == "section" && !import_section_wrapper.is_empty() {
                        writeln!(writer, "{}", import_section_wrapper)?;
                        import_section_wrapper = "";
                    }
                }
                "block" => {
                    if operation == "block" && !import_block_wrapper.is_empty() {
                        writeln!(writer, "{}", import_block_wrapper)?;
                        import_block_wrapper = "";
                    }
                }
                "wrapper" => {
                    if operation == "wrapper" && !import_text_wrapper.is_empty() {
                        writeln!(writer, "{}", import_text_wrapper)?;
                        import_text_wrapper = "";
                    }
                }
                _ => (),
            }
        }
    }

    Ok(())
}

fn add_import_sdk<W: Write>(writer: &mut W) -> std::io::Result<()> {
    let import_statement = "import { TextMenu, LinkMenu, ColorMenu, BlockEditMenu, IconMenu, CustomColorMenu, ButtonMenu, ImagePopup } from \"#spark-admin-sdk\";\n";
    writeln!(writer, "{}", import_statement)?;

    Ok(())
}

fn extract_tags(line: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let tag_regex = Regex::new(r"</?[^>]+>").unwrap();

    for capture in tag_regex.captures_iter(line) {
        tags.push(capture.get(0).unwrap().as_str());
    }

    tags
}

fn clean_tag(tag: &str) -> &str {
    let result = tag.trim_matches(|c| c == '<' || c == '>' || c == '/');

    result.split_whitespace().next().unwrap_or("")
}

fn modify_file_with_section(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import(&mut writer, lines.clone(), "section")?;

    let mut l: usize = 0;
    while l < lines.len() {
        let line = &lines[l];
        let start = leading_spaces(&lines[l]);
        let (is_editable, variant, tag) = check(line);
        if is_editable {
            let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);
            match variant {
                "section" => {
                    *current_id += 1;
                    writeln!(writer, "{}<SectionWrapper {}>", " ".repeat(start), new_id)?;
                    let mut stack = Vec::new();
                    let mut current_tags = extract_tags(line);

                    if !current_tags.is_empty() {
                        stack.push(clean_tag(current_tags[0]));
                    }

                    let mut starting_point: usize = 1;
                    while l < lines.len() && !stack.is_empty() {
                        for tag in &current_tags[starting_point..] {
                            if tag.ends_with("/>") {
                                continue;
                            }

                            if !tag.contains("/") {
                                stack.push(clean_tag(tag));
                            }
                            else{
                                if let Some(opening_tag) = stack.pop() {
                                    if opening_tag != clean_tag(tag) {
                                        panic!(
                                            "Mismatched tag in section at line {}. Expected </{}>, found </{}>.",
                                            l + 1, opening_tag, clean_tag(tag)
                                        );
                                    }
                                } else {
                                    panic!(
                                        "Unmatched closing tag in section {} at line {}.",
                                        tag, l + 1
                                    );
                                }
                            }
                        }
                        starting_point = 0;
                        writeln!(writer, "  {}", lines[l])?;
                        l += 1;
                        current_tags = extract_tags(&lines[l]);
                    }
                    writeln!(writer, "{}</SectionWrapper>", " ".repeat(start))?;
                }
                _ => {
                    writeln!(writer, "{}", line)?;
                    l += 1;
                }
            }
        } else {
            writeln!(writer, "{}", line)?;
            l += 1;
        }
    }

    writer.flush()?;
    rename(temp_file_path, file_path)?;

    Ok(())
}
fn modify_file_with_block(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import(&mut writer, lines.clone(), "block")?;

    let mut l: usize = 0;
    while l < lines.len() {
        let line = &lines[l];
        let start = leading_spaces(&lines[l]);
        let (is_editable, variant, tag) = check(line);
        if is_editable {
            let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);
            match variant {
                "block" => {
                    *current_id += 1;
                    writeln!(writer, "{}<BlockWrapper {}>", " ".repeat(start), new_id)?;
                    let mut stack = Vec::new();
                    let mut current_tags = extract_tags(line);

                    if !current_tags.is_empty() {
                        stack.push(clean_tag(current_tags[0]));
                    }

                    let mut starting_point: usize = 1;
                    while l < lines.len() && !stack.is_empty() {
                        for tag in &current_tags[starting_point..] {
                            if tag.ends_with("/>") {
                                continue;
                            }

                            if !tag.contains("/") {
                                stack.push(clean_tag(tag));
                            }
                            else{
                                if let Some(opening_tag) = stack.pop() {
                                    if opening_tag != clean_tag(tag) {
                                        panic!(
                                            "Mismatched tag in block at line {}. Expected </{}>, found </{}>.",
                                            l + 1, opening_tag, clean_tag(tag)
                                        );
                                    }
                                } else {
                                    panic!(
                                        "Unmatched closing in section tag {} at line {}.",
                                        tag, l + 1
                                    );
                                }
                            }
                        }
                        starting_point = 0;
                        writeln!(writer, "  {}", lines[l])?;
                        l += 1;
                        current_tags = extract_tags(&lines[l]);
                    }
                    writeln!(writer, "{}</BlockWrapper>", " ".repeat(start))?;
                }
                _ => {
                    writeln!(writer, "{}", line)?;
                    l += 1;
                }
            }
        } else {
            writeln!(writer, "{}", line)?;
            l += 1;
        }
    }

    writer.flush()?;
    rename(temp_file_path, file_path)?;

    Ok(())
}
fn modify_file_with_image(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import(&mut writer, lines.clone(), "image")?;

    let mut l: usize = 0;
    while l < lines.len() {
        let line = &lines[l];
        let start = leading_spaces(&lines[l]);
        let (is_editable, variant, tag) = check(line);
        if is_editable {
            let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);
            match variant {
                "image" => {
                    *current_id += 1;
                    writeln!(writer, "{}<ImageWrapper {}>", " ".repeat(start), new_id)?;
                    while l < lines.len() && !lines[l].contains(">") {
                        writeln!(writer, "  {}", lines[l])?;
                        l += 1;
                    }
                    writeln!(writer, "  {}", lines[l])?;
                    l += 1;
                    writeln!(writer, "{}</ImageWrapper>", " ".repeat(start))?;
                }
                _ => {
                    writeln!(writer, "{}", line)?;
                    l += 1;
                }
            }
        } else {
            writeln!(writer, "{}", line)?;
            l += 1;
        }
    }

    writer.flush()?;
    rename(temp_file_path, file_path)?;

    Ok(())
}
fn modify_file_with_wrapper(file_path: &str, name: &str, website_id: &str, current_id: &mut i32) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import(&mut writer, lines.clone(), "wrapper")?;

    let mut l: usize = 0;
    while l < lines.len() {
        let line = &lines[l];
        let start = leading_spaces(&lines[l]);
        let (is_editable, variant, tag) = check(line);
        if is_editable {
            let new_id = format!(r#"id="{name}_editable_{website_id}_{current_id}""#);
            match variant {
                "wrapper" => {
                    *current_id += 1;
                    writeln!(writer, "{}<Wrapper {}>", " ".repeat(start), new_id)?;
                    while l < lines.len() && !lines[l].contains(&format!("</{}>", tag)) {
                        writeln!(writer, "{}", lines[l])?;
                        l += 1;
                    }
                    writeln!(writer, "  {}", lines[l])?;
                    l += 1;
                    writeln!(writer, "{}</Wrapper>", " ".repeat(start))?;
                }
                _ => {
                    writeln!(writer, "{}", line)?;
                    l += 1;
                }
            }
        } else {
            writeln!(writer, "{}", line)?;
            l += 1;
        }
    }

    writer.flush()?;
    rename(temp_file_path, file_path)?;

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

fn modify_page(file_path: &str) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let temp_file_path = format!("{file_path}.tmp");
    let temp_file = File::create(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    add_import_sdk(&mut writer)?;

    let mut l: usize = 0;
    let mut found: bool = false;
    while l < lines.len() {
        let line = &lines[l];
        writeln!(writer, "{}", line)?;
        if !found && (EDITABLE_TAGS.iter().any(|&tag| line.contains(&format!("</{}>", tag))) || line.contains("<>")) {
            let start = leading_spaces(&line)+1;
            writeln!(writer, "{}<TextMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<LinkMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<ColorMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<BlockEditMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<CustomColorMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<ButtonMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<IconMenu />", " ".repeat(start))?;
            writeln!(writer, "{}<ImagePopup />", " ".repeat(start))?;
            found = true;
        }
        l += 1;
    }

    writer.flush()?;
    rename(temp_file_path, file_path)?;

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
                    modify_file_with_section(file_path.to_str().unwrap(), name, website_id, &mut current_id)?;
                    modify_file_with_block(file_path.to_str().unwrap(), name, website_id, &mut current_id)?;
                    modify_file_with_image(file_path.to_str().unwrap(), name, website_id, &mut current_id)?;
                    modify_file_with_wrapper(file_path.to_str().unwrap(), name, website_id, &mut current_id)?;
                }
            }
        }
        if entry.file_type().is_dir() {
            let dir_path = entry.path();
            if dir_path.ends_with("pages") {
                for inner_entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
                    if inner_entry.file_type().is_file() {
                        let file_path = inner_entry.path();
                        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                            if extension == "jsx" || extension == "tsx" {
                                modify_page(file_path.to_str().unwrap())?;
                            }
                        }
                    }
                }
            }
        }
    }

    // run_build(root_dir)?;

    Ok(())
}