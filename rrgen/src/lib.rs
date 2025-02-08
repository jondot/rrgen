use std::path::{Path, PathBuf};
use crate::MatchPositions::{All, First, Last};
use regex::Regex;
use serde::Deserialize;
use tera::{Context, Tera};

mod tera_filters;
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

#[derive(Deserialize, Debug, Default)]
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

#[derive(Deserialize, Debug, Default)]
struct Injection {
    into: String,
    content: String,

    #[serde(default)]
    inline: bool,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    skip_if: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    before: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    before_all: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    before_last: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    after: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    after_all: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    after_last: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    remove_lines: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    replace: Option<Regex>,

    #[serde(with = "serde_regex")]
    #[serde(default)]
    replace_all: Option<Regex>,

    #[serde(default)]
    prepend: bool,

    #[serde(default)]
    append: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Tera(#[from] tera::Error),
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

#[derive(Debug)]
pub enum GenResult {
    Skipped,
    Generated { message: Option<String> },
}

fn parse_template(input: &str) -> Result<(FrontMatter, String)> {
    // normalize line endings
    let input = input.replace("\r\n", "\n");

    let (fm, body) = input.split_once("---\n").ok_or_else(|| {
        Error::Message("cannot split document to frontmatter and body".to_string())
    })?;
    let frontmatter: FrontMatter = serde_yaml::from_str(fm)?;
    Ok((frontmatter, body.to_string()))
}
pub struct RRgen {
    working_dir: Option<PathBuf>,
    fs: Box<dyn FsDriver>,
    printer: Box<dyn Printer>,
    template_engine: Tera,
}

impl Default for RRgen {
    fn default() -> Self {
        let mut tera = Tera::default();
        tera_filters::register_all(&mut tera);
        Self {
            working_dir: None,
            fs: Box::new(RealFsDriver {}),
            printer: Box::new(ConsolePrinter {}),
            template_engine: tera,
        }
    }
}

impl RRgen {
    /// Creates a new [`RRgen`] instance with the specified working directory.
    ///
    /// # Example
    /// ```rust
    /// use rrgen::RRgen;
    ///
    /// let rgen = RRgen::with_working_dir("path");
    ///
    /// ```
    #[must_use]
    pub fn with_working_dir<P: AsRef<Path>>(path: P) -> Self {
        Self {
            working_dir: Some(path.as_ref().to_path_buf()),
            ..Default::default()
        }
    }

    /// Adds a custom template engine to the generator.
    ///
    /// ```rust
    /// use rrgen::RRgen;
    /// use tera::Tera;
    ///
    /// let mut tera = Tera::default();
    /// let rgen = RRgen::default().add_template_engine(tera);
    ///
    /// ```
    #[must_use]
    pub fn add_template_engine(self, mut template_engine: Tera) -> Self {
        tera_filters::register_all(&mut template_engine);
        Self {
            template_engine,
            ..self
        }
    }

    /// Generate from a template contained in `input`
    ///
    /// # Errors
    ///
    /// This function will return an error if operation fails
    pub fn generate(&self, input: &str, vars: &serde_json::Value) -> Result<GenResult> {
        let mut tera: Tera = self.template_engine.clone();
        let rendered = tera.render_str(input, &Context::from_serialize(vars.clone())?)?;
        let (frontmatter, body) = parse_template(&rendered)?;

        let path_to = if let Some(working_dir) = &self.working_dir {
            working_dir.join(frontmatter.to)
        } else {
            PathBuf::from(&frontmatter.to)
        };

        if frontmatter.skip_exists && self.fs.exists(&path_to) {
            self.printer.skip_exists(&path_to);
            return Ok(GenResult::Skipped);
        }
        if let Some(skip_glob) = frontmatter.skip_glob {
            if glob::glob(&skip_glob)?.count() > 0 {
                self.printer.skip_exists(&path_to);
                return Ok(GenResult::Skipped);
            }
        }

        if self.fs.exists(&path_to) {
            self.printer.overwrite_file(&path_to);
        } else {
            self.printer.add_file(&path_to);
        }
        // write main file
        self.fs.write_file(&path_to, &body)?;

        // handle injects
        if let Some(injections) = frontmatter.injections {
            for injection in &injections {
                let injection_to = self.working_dir.as_ref().map_or_else(
                    || PathBuf::from(&injection.into),
                    |working_dir| working_dir.join(&injection.into),
                );
                if !self.fs.exists(&injection_to) {
                    return Err(Error::Message(format!(
                        "cannot inject into {}: file does not exist",
                        injection.into,
                    )));
                }

                let file_content = self.fs.read_file(&injection_to)?;
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
                    insert_content_at_matches(&file_content, content, injection.inline, before, First, InsertionPoint::Before)
                } else if let Some(before_last) = &injection.before_last {
                    insert_content_at_matches(&file_content, content, injection.inline, before_last, Last, InsertionPoint::Before)
                } else if let Some(before_last) = &injection.before_all {
                    insert_content_at_matches(&file_content, content, injection.inline, before_last, All, InsertionPoint::Before)
                } else if let Some(after) = &injection.after {
                    insert_content_at_matches(&file_content, content, injection.inline, after, First, InsertionPoint::After)
                } else if let Some(after_last) = &injection.after_last {
                    insert_content_at_matches(&file_content, content, injection.inline, after_last, Last, InsertionPoint::After)
                } else if let Some(after_all) = &injection.after_all {
                    insert_content_at_matches(&file_content, content, injection.inline, after_all, All, InsertionPoint::After)
                } else if let Some(remove_lines) = &injection.remove_lines {
                    let lines = file_content
                        .lines()
                        .filter(|line| !remove_lines.is_match(line))
                        .collect::<Vec<_>>();
                    lines.join("\n")
                } else if let Some(replace) = &injection.replace {
                    replace
                        .replace(&file_content, content.as_str())
                        .to_string()
                } else if let Some(replace) = &injection.replace_all {
                    replace
                        .replace_all(&file_content, content.as_str())
                        .to_string()
                } else {
                    println!("warning: no injection made");
                    file_content.clone()
                };

                self.fs.write_file(&injection_to, &new_content)?;
                self.printer.injected(&injection_to);
            }
        }
        Ok(GenResult::Generated {
            message: frontmatter.message.clone(),
        })
    }
}
#[derive(Debug, Clone)]
enum MatchPositions {
    All,
    First,
    Last,
}

/// Finds the positions of lines in the file content that match the provided regex pattern.
///
/// # Arguments
///
/// * `lines` - A vector of lines from the file content.
/// * `regex` - The regex pattern to match lines.
/// * `input` - Specifies whether to match all, first, or last occurrences.
///
/// # Returns
///
/// A vector of indices representing the positions of the matching lines.
fn find_positions(lines: Vec<&str>, regex: &Regex, input: &MatchPositions) -> Vec<usize> {
    let matching_positions: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| if regex.is_match(line) { Some(i) } else { None })
        .collect();

    match input {
        All => matching_positions,
        First => matching_positions.into_iter().take(1).collect(),
        Last => matching_positions.into_iter().rev().take(1).collect(),
    }
}
#[derive(Debug)]
enum InsertionPoint {
    Before,
    After,
}

/// Inserts content at specified positions in the file content based on the provided regex pattern.
///
/// # Arguments
///
/// * `file_content` - The original content of the file.
/// * `content` - The content to be inserted.
/// * `inline` - Whether to insert the content inline or as a new line.
/// * `regex` - The regex pattern to match positions for insertion.
/// * `match_positions` - Specifies whether to match all, first, or last occurrences.
/// * `position` - Specifies whether to insert the content before or after the matched positions.
///
/// # Returns
///
/// A new string with the content inserted at the specified positions.
fn insert_content_at_matches(
    file_content: &str,
    content: &str,
    inline: bool,
    regex: &Regex,
    match_positions: MatchPositions,
    position: InsertionPoint,
) -> String {
    let lines = file_content.lines().collect::<Vec<_>>();
    let positions = find_positions(lines.clone(), regex, &match_positions);

    let replace_with = |caps: &regex::Captures| {
        match position {
            InsertionPoint::Before => format!("{}{}", content, &caps[0]),
            InsertionPoint::After => format!("{}{}", &caps[0], content),
        }
    };

    let new_lines = lines.iter()
        .enumerate()
        .flat_map(|(index,line)| {
        if regex.is_match(line) && positions.contains(&index) {
            if inline {
                let new_line = match match_positions {
                    All => regex.replace_all(line, replace_with).to_string(),
                    First => regex.replace(line, replace_with).to_string(),
                    Last => {
                        let count = regex.find_iter(line).count();
                        regex.replacen(line, count, replace_with).to_string()
                    }
                };
                vec![new_line]
            } else {
                if matches!(position, InsertionPoint::Before) {
                    vec![content.to_string(), line.to_string()]
                } else {
                    vec![line.to_string(), content.to_string()]
                }
            }
        } else {
            vec![line.to_string()]
        }
    }).collect::<Vec<String>>();
    new_lines.join("\n")
}