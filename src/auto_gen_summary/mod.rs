use hex;
use md5::{Digest, Md5};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::MDBook;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::auto_gen_summary::config::{AutoGenConfig, DirectoryWithoutIndexBehavior};

pub mod config;

pub const PREPROCESSOR_NAME: &str = "auto-gen-summary";
const SUMMARY_FILE: &str = "SUMMARY.md";

pub struct MdEntry {
    title: String,
    /// The link that the entry link point to. None corresponds to a draft entry.
    path: Option<PathBuf>,
    /// A path used only for sorting. Must not be empty.
    sorting_path: PathBuf,
    children: Vec<MdEntry>,
}

pub struct AutoGenSummary;

impl AutoGenSummary {
    pub fn new() -> AutoGenSummary {
        AutoGenSummary
    }
}

impl Preprocessor for AutoGenSummary {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, _book: Book) -> Result<Book, Error> {
        let mut config = AutoGenConfig::new();
        config.apply_config(&ctx.config)?;

        let source_dir = ctx.root.join(&ctx.config.book.src);

        gen_summary(&source_dir, &config);

        match MDBook::load(&ctx.root) {
            Ok(mdbook) => Ok(mdbook.book),
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn md5(buf: &String) -> String {
    let mut hasher = Md5::new();
    hasher.update(buf.as_bytes());
    let f = hasher.finalize();
    let md5_vec = f.as_slice();
    let md5_string = hex::encode_upper(md5_vec);

    return md5_string;
}

pub fn gen_summary(source_dir: &Path, config: &AutoGenConfig) {
    let group = walk_dir(source_dir, config);
    let mut lines = vec![String::from("# Summary\n")];

    if let Some(mut group) = group {
        sort_entry_recursive(&mut group);

        lines.push(generate_summary_line(
            0,
            &group.title,
            &group
                .path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(String::from("")),
        ));

        let mut last_was_dir = false;

        for child in group.children {
            let entry_name = if let Some(path) = &child.path {
                path.file_name().map(|p| OsString::from(p))
            } else {
                None
            };
            let is_dir = child.children.len() > 0;

            if !is_dir && entry_name == Some(OsString::from(SUMMARY_FILE)) {
                continue; // filter out summary file in first level directory
            }

            if last_was_dir || is_dir {
                lines.push(String::from("\n----\n"));
            }
            last_was_dir = is_dir;

            lines.append(&mut gen_summary_for_entry(source_dir, 0, &child, config));
        }

        // lines.append(&mut gen_summary_for_entry(source_dir, &group, config));
    } else {
        eprintln!("Warn: Your root directory is not being recognized. Make sure you have an index file (default 'SUMMARY.md')");
    }

    let buff: String = lines.join("\n");

    let new_md5_string = md5(&buff);

    let mut summary_file_path = PathBuf::from(source_dir);
    summary_file_path.push(SUMMARY_FILE);

    let summary_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&summary_file_path)
        .unwrap();

    let mut old_summary_file_content = String::new();
    let mut summary_file_reader = BufReader::new(summary_file);
    summary_file_reader
        .read_to_string(&mut old_summary_file_content)
        .unwrap();

    let old_md5_string = md5(&old_summary_file_content);

    if new_md5_string == old_md5_string {
        return;
    }

    let summary_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(&summary_file_path)
        .unwrap();
    let mut summary_file_writer = BufWriter::new(summary_file);
    summary_file_writer.write_all(buff.as_bytes()).unwrap();
}

/// Recursively sorts the entries by path
pub fn sort_entry_recursive(entry: &mut MdEntry) {
    entry
        .children
        .sort_by(|a, b| a.sorting_path.cmp(&b.sorting_path));

    for mut child in &mut entry.children {
        sort_entry_recursive(&mut child);
    }
}

fn gen_summary_for_entry(
    root_dir: &Path,
    depth: usize,
    md_entry: &MdEntry,
    config: &AutoGenConfig,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    let path = if let Some(path) = &md_entry.path {
        path.strip_prefix(root_dir)
            .unwrap()
            .to_string_lossy()
            .to_string()
    } else {
        String::from("")
    };

    lines.push(generate_summary_line(depth, &md_entry.title, &path));

    for child in &md_entry.children {
        let mut line = gen_summary_for_entry(root_dir, depth + 1, &child, config);
        lines.append(&mut line);
    }

    lines
}

fn generate_summary_line(indentation_level: usize, title: &str, link: &str) -> String {
    format!(
        "{}* [{}]({})",
        " ".repeat(4 * indentation_level),
        title,
        link
    )
}

fn get_title(entry: &DirEntry) -> String {
    let md_file = std::fs::File::open(entry.path().to_str().unwrap()).unwrap();
    let mut md_file_content = String::new();
    let mut md_file_reader = BufReader::new(md_file);
    md_file_reader.read_to_string(&mut md_file_content).unwrap();
    let lines = md_file_content.split("\n");

    let mut title: String = "".to_string();
    let mut first_h1_line = "";
    for line in lines {
        if line.starts_with("# ") {
            first_h1_line = line.trim_matches('#').trim();
            break;
        }
    }

    if first_h1_line.len() > 0 {
        title = first_h1_line.to_string();
    }

    return title;
}

fn walk_dir(dir: &Path, config: &AutoGenConfig) -> Option<MdEntry> {
    let read_dir = fs::read_dir(dir).unwrap();

    let mut child_directories = Vec::new();
    let mut result_children = Vec::new();
    let mut index_entry = None;

    for entry in read_dir {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            child_directories.push(entry);
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap().to_string();
        if config.directory_index_names.contains(&file_name) {
            let _ = index_entry.insert(entry);
            continue;
        }

        if entry.path().extension() != Some(&OsString::from("md")) {
            continue;
        }

        let title = get_title(&entry);

        let md = MdEntry {
            title: if config.first_line_as_link_text && title.len() > 0 {
                title
            } else {
                file_name.to_string()
            },
            path: Some(entry.path()),
            sorting_path: entry.path(),
            children: Vec::new(),
        };

        result_children.push(md);
    }

    if index_entry.is_none()
        && config.directory_without_index_behavior == DirectoryWithoutIndexBehavior::Ignore
    {
        return None;
    }

    for child_dir in child_directories {
        let g = walk_dir(&child_dir.path(), &config);
        if let Some(g) = g {
            result_children.push(g);
        }
    }

    let dir_name_as_string = dir.file_name().unwrap().to_string_lossy().to_string();

    Some(match index_entry {
        Some(index_entry) => MdEntry {
            title: {
                let t = get_title(&index_entry);
                if t.len() > 0 {
                    t
                } else {
                    dir_name_as_string
                }
            },
            path: Some(PathBuf::from(index_entry.path())),
            sorting_path: PathBuf::from(dir),
            children: result_children,
        },
        None => MdEntry {
            title: dir_name_as_string,
            path: None,
            sorting_path: PathBuf::from(dir),
            children: result_children,
        },
    })
}
