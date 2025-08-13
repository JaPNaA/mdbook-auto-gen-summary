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

use crate::auto_gen_summary::config::AutoGenConfig;

pub mod config;

pub const PREPROCESSOR_NAME: &str = "auto-gen-summary";
const SUMMARY_FILE: &str = "SUMMARY.md";

#[derive(Debug)]
pub struct MdFile {
    pub title: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct MdGroup {
    pub title: String,
    pub path: PathBuf,
    pub readme_name: Option<String>,
    pub group_list: Vec<MdGroup>,
    pub md_list: Vec<MdFile>,
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
    let lines = gen_summary_lines(source_dir, &group, config);
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

fn gen_summary_lines(root_dir: &Path, group: &MdGroup, config: &AutoGenConfig) -> Vec<String> {
    let mut lines: Vec<String> = vec![];

    let path = group.path.strip_prefix(root_dir).unwrap();
    let cnt = path.components().count();

    let buff_spaces = if cnt > 0 {
        String::from(" ".repeat(4 * (cnt - 1)))
    } else {
        String::from("")
    };

    let buff_link: String;
    if cnt == 0 {
        lines.push(String::from("# Summary"));

        buff_link = format!(
            "{}* [{}]({})",
            buff_spaces,
            group.title,
            group
                .readme_name
                .as_ref()
                .expect("Internal error: no README file for directory")
        );
    } else {
        buff_link = format!(
            "{}* [{}]({}/{})",
            buff_spaces,
            group.title,
            path.to_string_lossy().to_string(),
            group
                .readme_name
                .as_ref()
                .expect("Internal error: no README file for directory")
        );
    }

    // Insert a horizontal line before first-level directories
    if cnt == 1 {
        lines.push(String::from("\n----\n"));
    }

    lines.push(buff_link);

    for md in &group.md_list {
        let path = md.path.strip_prefix(root_dir).unwrap();
        if path.file_name() == Some(&OsString::from(SUMMARY_FILE)) {
            continue;
        }

        if config
            .directory_index_names
            .contains(&match path.file_name() {
                Some(x) => x.to_string_lossy().to_string(),
                None => String::from(""),
            })
        {
            continue;
        }

        let cnt = path.components().count();
        let buff_spaces = String::from(" ".repeat(4 * (cnt - 1)));

        let buff_link = format!(
            "{}* [{}]({})",
            buff_spaces,
            &md.title,
            path.to_string_lossy().to_string()
        );

        lines.push(buff_link);
    }

    for group in &group.group_list {
        let mut line = gen_summary_lines(root_dir, group, config);
        lines.append(&mut line);
    }

    lines
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

fn walk_dir(dir: &Path, config: &AutoGenConfig) -> MdGroup {
    let read_dir = fs::read_dir(dir).unwrap();

    let mut group = MdGroup {
        title: dir.file_name().unwrap().to_string_lossy().to_string(),
        path: PathBuf::from(dir),
        readme_name: None,
        group_list: vec![],
        md_list: vec![],
    };

    for entry in read_dir {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            let g = walk_dir(&entry.path(), &config);
            if g.readme_name.is_some() {
                group.group_list.push(g);
            }
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap().to_string();
        if config.directory_index_names.contains(&file_name) {
            let _ = group.readme_name.insert(file_name.clone());
        }
        let arr: Vec<&str> = file_name.split(".").collect();
        if arr.len() < 2 {
            continue;
        }

        if entry.path().extension() != Some(&OsString::from("md")) {
            continue;
        }

        let title = get_title(&entry);

        let md = MdFile {
            title: if config.first_line_as_link_text && title.len() > 0 {
                title
            } else {
                file_name.to_string()
            },
            path: entry.path(),
        };

        group.md_list.push(md);
    }

    return group;
}
