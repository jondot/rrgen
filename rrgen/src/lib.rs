use std::path::{Path, PathBuf};

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
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    Message(String),

    /// An injection named a file that is not there.
    #[error("cannot inject into `{path}`: file does not exist")]
    InjectionTargetMissing { path: String },

    /// An injection's anchor pattern matched no line in the target file.
    ///
    /// This used to be silent: the file was rewritten unchanged and reported as
    /// injected. That is how a generator comes to report success while leaving
    /// the code it generated unreachable — a migration that never runs, a route
    /// that is never mounted — with nothing in the output to say so.
    #[error(
        "cannot inject into `{path}`: no line matches the `{strategy}` pattern `{pattern}`\n\n\
         Nothing was written, so this was not added:\n\n{content}\n\n\
         Restore a line matching that pattern in `{path}` and run this again, or \
         add the content by hand."
    )]
    InjectionAnchorNotFound {
        path: String,
        strategy: &'static str,
        pattern: String,
        content: String,
    },

    /// An injection declared content but no way to place it.
    #[error(
        "cannot inject into `{path}`: the injection says where to write but not where to put it \
         — expected one of `before`, `before_last`, `after`, `after_last`, `prepend`, `append`, \
         or `remove_lines`"
    )]
    InjectionHasNoPlacement { path: String },

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

/// Applies one injection to a file's contents.
///
/// Returns `Ok(None)` when the file is already in the state the injection asks
/// for and should be left alone. Every other outcome is either the new contents
/// or an error — an injection never quietly does nothing.
fn apply_injection(injection: &Injection, file_content: &str) -> Result<Option<String>> {
    let content = &injection.content;

    let new_content = if injection.prepend {
        format!("{content}\n{file_content}")
    } else if injection.append {
        format!("{file_content}\n{content}")
    } else if let Some(before) = &injection.before {
        insert(injection, file_content, "before", |lines| {
            lines.iter().position(|ln| before.is_match(ln))
        })?
    } else if let Some(before_last) = &injection.before_last {
        insert(injection, file_content, "before_last", |lines| {
            lines.iter().rposition(|ln| before_last.is_match(ln))
        })?
    } else if let Some(after) = &injection.after {
        insert(injection, file_content, "after", |lines| {
            lines
                .iter()
                .position(|ln| after.is_match(ln))
                .map(|p| p + 1)
        })?
    } else if let Some(after_last) = &injection.after_last {
        insert(injection, file_content, "after_last", |lines| {
            lines
                .iter()
                .rposition(|ln| after_last.is_match(ln))
                .map(|p| p + 1)
        })?
    } else if let Some(remove_lines) = &injection.remove_lines {
        let kept = file_content
            .lines()
            .filter(|line| !remove_lines.is_match(line))
            .collect::<Vec<_>>();
        if kept.len() == file_content.lines().count() {
            return Ok(None);
        }
        kept.join("\n")
    } else {
        return Err(Error::InjectionHasNoPlacement {
            path: injection.into.clone(),
        });
    };

    Ok(Some(keep_trailing_newline(file_content, new_content)))
}

/// Inserts the injection's content at the line index `locate` picks.
///
/// Fails when `locate` finds nothing. That case is the whole reason this
/// function exists: an anchor that no longer matches means the generated code
/// is not wired up, and the only chance to say so is here.
fn insert(
    injection: &Injection,
    file_content: &str,
    strategy: &'static str,
    locate: impl Fn(&[&str]) -> Option<usize>,
) -> Result<String> {
    let mut lines = file_content.lines().collect::<Vec<_>>();
    let pos = locate(&lines).ok_or_else(|| Error::InjectionAnchorNotFound {
        path: injection.into.clone(),
        strategy,
        pattern: injection.pattern(strategy),
        content: indent(&injection.content),
    })?;
    lines.insert(pos, &injection.content);
    Ok(lines.join("\n"))
}

/// `str::lines` drops a trailing newline, so rejoining silently strips it from
/// every file an injection touches — leaving `\ No newline at end of file` in
/// the diff of code the generator wrote on the user's behalf.
fn keep_trailing_newline(original: &str, mut new: String) -> String {
    if original.ends_with('\n') && !new.ends_with('\n') {
        new.push('\n');
    }
    new
}

fn indent(content: &str) -> String {
    content
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

impl Injection {
    /// The source text of the anchor pattern for `strategy`, for error messages.
    fn pattern(&self, strategy: &str) -> String {
        let pattern = match strategy {
            "before" => self.before.as_ref(),
            "before_last" => self.before_last.as_ref(),
            "after" => self.after.as_ref(),
            "after_last" => self.after_last.as_ref(),
            _ => None,
        };
        pattern.map_or_else(|| "<none>".to_string(), ToString::to_string)
    }
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
            // Resolve against the working dir, like every other path in the
            // template. Globbing the process's current directory instead means
            // the skip silently never fires for a caller that set one.
            let skip_glob = self.working_dir.as_ref().map_or(skip_glob.clone(), |dir| {
                dir.join(&skip_glob).to_string_lossy().into_owned()
            });
            if glob::glob(&skip_glob)?.count() > 0 {
                self.printer.skip_exists(&path_to);
                return Ok(GenResult::Skipped);
            }
        }

        // Work every injection out before writing anything.
        //
        // An injection can now fail, and failing after the main file is written
        // leaves the caller stuck: the file exists, so a second run hits
        // `skip_exists`/`skip_glob` and returns before it reaches the injection
        // that never happened. Deciding everything first makes a failed
        // generation a no-op you can simply re-run.
        let pending = self.plan_injections(frontmatter.injections.as_deref())?;

        if self.fs.exists(&path_to) {
            self.printer.overwrite_file(&path_to);
        } else {
            self.printer.add_file(&path_to);
        }
        // write main file
        self.fs.write_file(&path_to, &body)?;

        for (path, content) in pending {
            self.fs.write_file(&path, &content)?;
            self.printer.injected(&path);
        }

        Ok(GenResult::Generated {
            message: frontmatter.message.clone(),
        })
    }

    /// Resolves every injection to the file contents it would write.
    ///
    /// Touches nothing on disk. Returns the writes in template order.
    fn plan_injections(&self, injections: Option<&[Injection]>) -> Result<Vec<(PathBuf, String)>> {
        let mut pending: Vec<(PathBuf, String)> = Vec::new();

        for injection in injections.unwrap_or_default() {
            let injection_to = self.working_dir.as_ref().map_or_else(
                || PathBuf::from(&injection.into),
                |working_dir| working_dir.join(&injection.into),
            );
            if !self.fs.exists(&injection_to) {
                return Err(Error::InjectionTargetMissing {
                    path: injection.into.clone(),
                });
            }

            // Later injections see earlier ones. Two injections into the same
            // file is ordinary — an import line and a registration line — and
            // computing both against the on-disk copy would lose the first.
            let file_content = match pending.iter().rev().find(|(path, _)| path == &injection_to) {
                Some((_, planned)) => planned.clone(),
                None => self.fs.read_file(&injection_to)?,
            };

            if let Some(skip_if) = &injection.skip_if {
                if skip_if.is_match(&file_content) {
                    continue;
                }
            }

            let Some(new_content) = apply_injection(injection, &file_content)? else {
                // A removal that matched nothing. The file is already in the
                // state the template asked for, so there is nothing to write
                // and nothing to report.
                continue;
            };

            pending.push((injection_to, new_content));
        }

        Ok(pending)
    }
}
