#[cfg(all(feature = "tera", feature = "minijinja"))]
compile_error!("You cannot enable both 'tera' and 'minijinja' at the same time.");

#[cfg(not(any(feature = "tera", feature = "minijinja")))]
compile_error!("You must enable exactly one feature: 'tera' or 'minijinja'.");

use std::path::Path;

use regex::Regex;
use serde::Deserialize;
#[cfg(feature = "tera")]
use tera::{Context, Tera};
#[cfg(feature = "minijinja")]
use minijinja::{Environment};
use log::debug;

#[cfg(feature = "tera")]
mod tera_filters;
#[cfg(feature = "minijinja")]
mod minijinja_filters;


pub trait FsDriver {
    /// Write a file
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;

    /// Read a file
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails
    fn read_file(&self, path: &Path) -> Result<String>;

    fn exists(&self, path: &Path) -> bool;
}

pub struct RealFsDriver {}
impl FsDriver for RealFsDriver {
    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        let dir = path.parent().expect("cannot get folder");
        if !dir.exists() {
            fs_err::create_dir_all(dir)?;
        }
        Ok(fs_err::write(path, content)?)
    }

    fn read_file(&self, path: &Path) -> Result<String> {
        Ok(fs_err::read_to_string(path)?)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

pub trait Printer {
    fn overwrite_file(&self, file_to: &Path);
    fn skip_exists(&self, file_to: &Path);
    fn add_file(&self, file_to: &Path);
    fn injected(&self, file_to: &Path);
}
pub struct ConsolePrinter {}
impl Printer for ConsolePrinter {
    fn overwrite_file(&self, file_to: &Path) {
        println!("overwritten: {file_to:?}");
    }

    fn add_file(&self, file_to: &Path) {
        println!("added: {file_to:?}");
    }

    fn injected(&self, file_to: &Path) {
        println!("injected: {file_to:?}");
    }

    fn skip_exists(&self, file_to: &Path) {
        println!("skipped (exists): {file_to:?}");
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
struct FrontMatter {
    to: String,

    #[serde(default)]
    skip_exists: bool,

    #[serde(default)]
    skip_glob: Option<String>,

    #[serde(default)]
    message: Option<String>,

    #[serde(default)]
    injections: Option<Vec<Injection>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct Injection {
    into: String,
    content: String,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    skip_if: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    before: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    before_last: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    after: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    after_last: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    remove_lines: Option<Regex>,

    #[serde(default)]
    prepend: bool,

    #[serde(default)]
    append: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[cfg(feature = "tera")]
    #[error(transparent)]
    Tera(#[from] tera::Error),
    #[cfg(feature = "minijinja")]
    #[error(transparent)]
    MiniJinja(#[from] minijinja::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    YAML(#[from] serde_yaml::Error),
    #[error(transparent)]
    Glob(#[from] glob::PatternError),
    #[error(transparent)]
    Any(Box<dyn std::error::Error + Send + Sync>),
}
type Result<T> = std::result::Result<T, Error>;

pub enum GenResult {
    Skipped,
    Generated { message: Option<String> },
}

/// Split `input` into chunks according to `---` separator and return a vec of pairs of frontmatter and body.
///
/// # Errors
///
/// This function will return an error if operation fails
fn parse_template(input: &str) -> Result<Vec<(FrontMatter, String)>> {
    let parts: Vec<&str> = input.split("---\n").filter(|&s| !s.trim().is_empty()).collect();

    let parts_split: Result<Vec<(FrontMatter, String)>> = parts.chunks(2)
        .map(|chunk| {
            if chunk.len() != 2 {
                return Err(Error::Message("cannot split document into frontmatter and body".to_string()));
            }
            let fm = chunk[0].trim();
            let body = chunk[1].trim();
            let front_matter: FrontMatter = serde_yaml::from_str(fm)?;
            Ok((front_matter, body.to_string()))
        })
        .collect();

    parts_split
}

pub struct RRgen {
    fs: Box<dyn FsDriver>,
    printer: Box<dyn Printer>,
    #[cfg(feature = "tera")]
    pub tera: Tera,
    #[cfg(feature = "minijinja")]
    pub minijinja: Environment<'static>,
}

impl Default for RRgen {
    fn default() -> Self {
        #[cfg(feature = "tera")]
        let mut tera_instance = Tera::default();
        #[cfg(feature = "tera")]
        tera_filters::register_all(&mut tera_instance);

        #[cfg(feature = "minijinja")]
        let mut minijinja = Environment::new();
        #[cfg(feature = "minijinja")]
        minijinja_filters::register_all(&mut minijinja);

        Self {
            fs: Box::new(RealFsDriver {}),
            printer: Box::new(ConsolePrinter {}),
            #[cfg(feature = "tera")]
            tera: tera_instance,
            #[cfg(feature = "minijinja")]
            minijinja,
        }
    }
}

impl RRgen {
    /// Generate from a template contained in `input`
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    pub fn generate(&mut self, input: &str, vars: &serde_json::Value) -> Result<GenResult> {
        debug!("input: {input:?}");
        debug!("vars: {vars:?}");
        #[cfg(feature = "tera")]{
            let rendered = self.tera.render_str(input, &Context::from_serialize(vars.clone())?)?;
            return self.handle_rendered(rendered);
        }
        #[cfg(feature = "minijinja")]{
            let rendered = self.minijinja.render_str(input, vars.clone())?;
            return self.handle_rendered(rendered);
        }
    }

    /// Generate from a template added in the template engine given by `name`
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    pub fn generate_by_template_with_name(&mut self, name: &str, vars: &serde_json::Value) -> Result<GenResult> {
        #[cfg(feature = "tera")]{
            let rendered = self.tera.render_str(name, &Context::from_serialize(vars.clone())?)?;
            return self.handle_rendered(rendered);
        }

        #[cfg(feature = "minijinja")]{
            let template = self.minijinja.get_template(name);
            let rendered = template?.render(vars)?;
            return self.handle_rendered(rendered);
        }
    }

    /// Add name and template in template engine
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    pub fn add_template(&mut self, name: &str, template: &str) -> Result<()> {

        #[cfg(feature = "tera")]{
            self.tera.add_raw_template(name, template)?
        }
        #[cfg(feature = "minijinja")]{
            self.minijinja.add_template_owned(name.to_string(), template.to_string())?
        }
        Ok(())
    }

    /// Handle rendered string by splitting to pairs of frontmatter and body and then handle frontmatter
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    fn handle_rendered(&mut self, rendered: String) -> Result<GenResult> {
        debug!("rendered: {rendered:?}");
        let parts = parse_template(&rendered)?;
        let messages: Vec<String> = parts.iter()
            .map(|(frontmatter, body)| self.handle_frontmatter_and_body(frontmatter.clone(), body.clone()))
            .collect::<Result<Vec<GenResult>>>()?
            .into_iter()
            .filter_map(|gen_result| {
                if let GenResult::Generated { message: Some(msg) } = gen_result {
                    Some(msg)
                } else {
                    None
                }
            })
            .collect();

        let merged_message = messages.join("\n");
        Ok(GenResult::Generated { message: Some(merged_message) })
    }

    /// Handle frontmatter and body
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    fn handle_frontmatter_and_body(&mut self, frontmatter: FrontMatter, body: String) -> Result<GenResult> {
        let path_to = Path::new(&frontmatter.to);

        if frontmatter.skip_exists && self.fs.exists(path_to) {
            self.printer.skip_exists(path_to);
            return Ok(GenResult::Skipped);
        }
        if let Some(skip_glob) = frontmatter.skip_glob {
            if glob::glob(&skip_glob)?.count() > 0 {
                self.printer.skip_exists(path_to);
                return Ok(GenResult::Skipped);
            }
        }

        if self.fs.exists(path_to) {
            self.printer.overwrite_file(path_to);
        } else {
            self.printer.add_file(path_to);
        }
        // write main file
        self.fs.write_file(path_to, &body)?;

        // handle injects
        self.handle_injects(frontmatter.injections, frontmatter.message.clone())?;
        Ok(GenResult::Generated {
            message: frontmatter.message.clone(),
        })
    }

    fn handle_injects(&self, injections: Option<Vec<Injection>>, message: Option<String>) -> Result<GenResult> {
        if let Some(injections) = injections {
            for injection in &injections {
                let injection_to = Path::new(&injection.into);
                if !self.fs.exists(injection_to) {
                    return Err(Error::Message(format!(
                        "cannot inject into {}: file does not exist",
                        injection.into,
                    )));
                }

                let file_content = self.fs.read_file(injection_to)?;
                let content = &injection.content;

                if let Some(skip_if) = &injection.skip_if {
                    if skip_if.is_match(&file_content) {
                        continue;
                    }
                }

                let new_content = if injection.prepend {
                    format!("{content}\n{file_content}")
                } else if injection.append {
                    format!("{file_content}\n{content}")
                } else if let Some(before) = &injection.before {
                    let mut lines = file_content.lines().collect::<Vec<_>>();
                    let pos = lines.iter().position(|ln| before.is_match(ln));
                    if let Some(pos) = pos {
                        lines.insert(pos, content);
                    }
                    lines.join("\n")
                } else if let Some(before_last) = &injection.before_last {
                    let mut lines = file_content.lines().collect::<Vec<_>>();
                    let pos = lines.iter().rposition(|ln| before_last.is_match(ln));
                    if let Some(pos) = pos {
                        lines.insert(pos, content);
                    }
                    lines.join("\n")
                } else if let Some(after) = &injection.after {
                    let mut lines = file_content.lines().collect::<Vec<_>>();
                    let pos = lines.iter().position(|ln| after.is_match(ln));
                    if let Some(pos) = pos {
                        lines.insert(pos + 1, content);
                    }
                    lines.join("\n")
                } else if let Some(after_last) = &injection.after_last {
                    let mut lines = file_content.lines().collect::<Vec<_>>();
                    let pos = lines.iter().rposition(|ln| after_last.is_match(ln));
                    if let Some(pos) = pos {
                        lines.insert(pos + 1, content);
                    }
                    lines.join("\n")
                } else if let Some(remove_lines) = &injection.remove_lines {
                    let lines = file_content
                        .lines()
                        .filter(|line| !remove_lines.is_match(line))
                        .collect::<Vec<_>>();
                    lines.join("\n")
                } else {
                    println!("warning: no injection made");
                    file_content.clone()
                };

                self.fs.write_file(injection_to, &new_content)?;
                self.printer.injected(injection_to);
            }
            Ok(GenResult::Generated {
                message: message.clone(),
            })
        } else {
            Ok(GenResult::Skipped)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template() {
        let input = r#"
---
to: file1.txt
message: "File file1.txt was created successfully."
---
print some content #1
---
to: file2.txt
message: "File file2.txt was created successfully."
---
print some content #2
"#;

        let result = parse_template(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0.to, "file1.txt");
        assert_eq!(result[0].1, "print some content #1");
        assert_eq!(result[1].0.to, "file2.txt");
        assert_eq!(result[1].1, "print some content #2");
    }

    #[test]
    fn test_single_header_template_with_split() {
        let input = r#"
---
to: file1.txt
message: "File file1.txt was created successfully."
---
print some content #1
"#;

        let result = parse_template(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.to, "file1.txt");
        assert_eq!(result[0].1, "print some content #1");
    }

    #[test]
    fn test_single_header_template_without_split() {
        let input = r#"
to: file1.txt
message: "File file1.txt was created successfully."
---
print some content #1
"#;

        let result = parse_template(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.to, "file1.txt");
        assert_eq!(result[0].1, "print some content #1");
    }


    #[test]
    fn test_parse_template_with_error() {
        let input = r#"
---
to: ./file1.txt
message: "File file1.txt was created successfully."
---
print some content
--- incomplete header
"#;

        let result = parse_template(input);
        assert!(result.is_err());
    }
}