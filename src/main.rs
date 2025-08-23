mod auto_gen_summary;

use auto_gen_summary::AutoGenSummary;
use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::process;

use crate::auto_gen_summary::config::{AutoGenConfig, DirectoryWithoutIndexBehavior};

pub fn make_app() -> App<'static, 'static> {
    App::new("auto-gen-summary-preprocessor")
        .about("A mdbook preprocessor to auto generate book summary")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            SubCommand::with_name("gen")
                .arg(
                    Arg::with_name("dir")
                        .required(true)
                        .help("A path to the mdbook src directory"),
                )
                .arg(
                    Arg::with_name("title")
                        .required(false)
                        .short("t")
                        .long("title")
                        .help("Use the first line of markdown files the title in SUMMARY.md"),
                )
                .arg(
                    Arg::with_name("dir-title")
                        .required(false)
                        .short("T")
                        .long("dir-title")
                        .help(
                            "Use the first line of directory index files the title in SUMMARY.md",
                        ),
                )
                .arg(
                    Arg::with_name("dir-index-names")
                        .required(false)
                        .short("i")
                        .long("dir-index-names")
                        .takes_value(true)
                        .use_delimiter(true)
                        .help("Name of files to use as a directory index"),
                )
                .arg(
                    Arg::with_name("dir-without-index-behavior")
                        .required(false)
                        .short("w")
                        .long("dir-without-index-behavior")
                        .takes_value(true)
                        .case_insensitive(true)
                        .possible_values(&["ignore", "draft", "generate-stub-index"])
                        .help("Behavior of a directory without an index file"),
                )
                .about("gen SUMMARY.md"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    let preprocessor = AutoGenSummary::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Some(sub_args) = matches.subcommand_matches("gen") {
        let source_dir = sub_args
            .value_of("dir")
            .expect("Required argument")
            .to_string();

        let mut config = AutoGenConfig::new();
        config.first_line_as_link_text = sub_args.is_present("title");
        config.index_first_line_as_directory_link_text = sub_args.is_present("dir-title");

        if let Some(behavior) = sub_args.value_of("dir-without-index-behavior") {
            config.directory_without_index_behavior =
                DirectoryWithoutIndexBehavior::from_str(behavior).unwrap();
        }

        if let Some(index_names) = sub_args.values_of("dir-index-names") {
            let mut directory_index_names = HashSet::new();
            let mut generated_directory_index_name = None;

            for item in index_names {
                directory_index_names.insert(String::from(item));

                if generated_directory_index_name.is_none() {
                    generated_directory_index_name = Some(String::from(item));
                }
            }

            let Some(generated_directory_index_name) = generated_directory_index_name else {
                eprintln!("Directory index names must not be empty.");
                process::exit(1);
            };

            config.generated_directory_index_name = generated_directory_index_name;
            config.directory_index_names = directory_index_names;
        }

        auto_gen_summary::gen_summary(&PathBuf::from(source_dir), &config);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
