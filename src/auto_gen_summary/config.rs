use std::collections::HashSet;

use anyhow::Error;
use mdbook::Config;

use crate::auto_gen_summary::PREPROCESSOR_NAME;

const README_FILE: &str = "README.md";

const OPT_FIRST_LINE_AS_LINK: &str = "first-line-as-link-text";
const OPT_INDEX_FIRST_LINE_AS_DIRECTORY_LINK: &str = "index-first-line-as-directory-link-text";
const OPT_DIR_WITHOUT_INDEX_BEHAVIOR: &str = "directory-without-index-behavior";
const OPT_DIRECTORY_INDEX_NAMES: &str = "directory-index-names";

pub struct AutoGenConfig {
    /// Whether the first line of the markdown file should be used
    /// as the file's title. If false, the title is name of the file.
    ///
    /// Note, the first line must be an h1 (start with `# ` to be
    /// recognized as the title.)
    ///
    /// Default: false
    pub first_line_as_link_text: bool,

    /// The names of the files that can serve as an index for a directory.
    /// For example, "README.md" or "index.md".
    ///
    /// Default: { "README.md" }
    pub directory_index_names: HashSet<String>,

    /// Whether the first line of the directory index markdown file
    /// should be used as the directory's title. If false, the title
    /// is the name of the directory.
    ///
    /// Default: false
    pub index_first_line_as_directory_link_text: bool,

    /// What to do if we find a directory without an index file in the
    /// directory?
    ///
    /// Options: Ignore the directory, mark the directory as a draft, or
    /// create the index file (index file name specified by
    /// `generated_directory_index_name` option).
    ///
    /// Default: Ignore
    pub directory_without_index_behavior: DirectoryWithoutIndexBehavior,

    /// The name of the index file to create in a directory without any other
    /// index files. This is specified by the first `directory_index_names`
    /// option specified.
    ///
    /// Default: "README.md"
    pub generated_directory_index_name: String,
}

impl AutoGenConfig {
    pub fn new() -> AutoGenConfig {
        AutoGenConfig {
            first_line_as_link_text: false,
            index_first_line_as_directory_link_text: false,
            directory_without_index_behavior: DirectoryWithoutIndexBehavior::Ignore,
            directory_index_names: {
                let mut s = HashSet::new();
                s.insert(String::from(README_FILE));
                s
            },
            generated_directory_index_name: String::from(README_FILE),
        }
    }

    /// Given the config object from mdbook, extract the relevant options
    /// for this preprocessor.
    pub fn apply_config(&mut self, mdbook_config: &Config) -> Result<(), Error> {
        let Some(cfg) = mdbook_config.get_preprocessor(PREPROCESSOR_NAME) else {
            return Ok(());
        };

        if let Some(v) = cfg.get(OPT_FIRST_LINE_AS_LINK) {
            self.first_line_as_link_text = v.as_bool().unwrap_or(false);
        }

        if let Some(v) = cfg.get(OPT_INDEX_FIRST_LINE_AS_DIRECTORY_LINK) {
            self.index_first_line_as_directory_link_text = v.as_bool().unwrap_or(false);
        }

        if let Some(v) = cfg.get(OPT_DIR_WITHOUT_INDEX_BEHAVIOR) {
            let Some(v) = v.as_str() else {
                anyhow::bail!(
                    "Config key '{}' must be a string",
                    OPT_DIR_WITHOUT_INDEX_BEHAVIOR
                );
            };
            let Some(v) = DirectoryWithoutIndexBehavior::from_str(v) else {
                anyhow::bail!(
                    "Config key '{}' must be one of 'ignore', 'draft', or 'generate-stub-index'",
                    OPT_DIR_WITHOUT_INDEX_BEHAVIOR
                );
            };
            self.directory_without_index_behavior = v;
        }

        if let Some(v) = cfg.get(OPT_DIRECTORY_INDEX_NAMES) {
            let mut directory_index_names = HashSet::new();
            let mut generated_directory_index_name = None;

            let Some(v) = v.as_array() else {
                anyhow::bail!(
                    "Config key '{}' must be an array.",
                    OPT_DIRECTORY_INDEX_NAMES
                );
            };
            for item in v {
                let Some(item) = item.as_str() else {
                    anyhow::bail!(
                        "Item in array for config key {} is not a string.",
                        OPT_DIRECTORY_INDEX_NAMES
                    );
                };
                directory_index_names.insert(String::from(item));

                if generated_directory_index_name.is_none() {
                    generated_directory_index_name = Some(String::from(item));
                }
            }

            let Some(generated_directory_index_name) = generated_directory_index_name else {
                anyhow::bail!(
                    "Config key {} must not be empty.",
                    OPT_DIRECTORY_INDEX_NAMES
                )
            };

            self.generated_directory_index_name = generated_directory_index_name;
            self.directory_index_names = directory_index_names;
        }

        Ok(())
    }
}

/// Define the behavior for a directory with markdown files
/// but no index markdown files found
#[derive(PartialEq)]
pub enum DirectoryWithoutIndexBehavior {
    /// Ignore directory completely (default)
    Ignore,
    /// Mark the directory as a draft
    Draft,
    /// Create an stub index file automatically
    GenerateStubIndex,
}

impl DirectoryWithoutIndexBehavior {
    pub fn from_str(s: &str) -> Option<DirectoryWithoutIndexBehavior> {
        match s {
            "ignore" => Some(DirectoryWithoutIndexBehavior::Ignore),
            "draft" => Some(DirectoryWithoutIndexBehavior::Draft),
            "generate-stub-index" => Some(DirectoryWithoutIndexBehavior::GenerateStubIndex),
            _ => None,
        }
    }
}
