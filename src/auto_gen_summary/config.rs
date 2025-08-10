use std::collections::HashSet;

use anyhow::Error;
use mdbook::Config;

use crate::auto_gen_summary::PREPROCESSOR_NAME;

const README_FILE: &str = "README.md";

const OPT_FIRST_LINE_AS_LINK: &str = "first-line-as-link-text";
const OPT_GENERATE_STUB_DIRECTORY_INDEX: &str = "generate-stub-directory-index";
const OPT_DIRECTORY_INDEX_NAMES: &str = "directory-index-names";

pub struct AutoGenConfig {
    pub first_line_as_link_text: bool,
    pub generate_stub_directory_index: bool,
    pub directory_index_names: HashSet<String>,
}

impl AutoGenConfig {
    pub fn new() -> AutoGenConfig {
        AutoGenConfig {
            first_line_as_link_text: false,
            generate_stub_directory_index: false,
            directory_index_names: {
                let mut s = HashSet::new();
                s.insert(String::from(README_FILE));
                s
            },
        }
    }

    pub fn apply_config(&mut self, mdbook_config: &Config) -> Result<(), Error> {
        let Some(cfg) = mdbook_config.get_preprocessor(PREPROCESSOR_NAME) else {
            return Ok(());
        };

        if let Some(v) = cfg.get(OPT_FIRST_LINE_AS_LINK) {
            self.first_line_as_link_text = v.as_bool().unwrap_or(false);
        }

        if let Some(v) = cfg.get(OPT_FIRST_LINE_AS_LINK) {
            self.first_line_as_link_text = v.as_bool().unwrap_or(false);
        }

        if let Some(v) = cfg.get(OPT_DIRECTORY_INDEX_NAMES) {
            let mut directory_index_names = HashSet::new();

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
            }

            if directory_index_names.len() == 0 {
                anyhow::bail!(
                    "Config key {} must not be empty.",
                    OPT_DIRECTORY_INDEX_NAMES
                )
            }

            self.directory_index_names = directory_index_names;
        }

        Ok(())
    }
}
