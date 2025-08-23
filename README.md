# mdbook-auto-gen-summary

A preprocessor and CLI tool for mdbook to automatically generate the `SUMMARY.md` file from your existing directory structure.

## Usage

### Install

```bash
cargo install mdbook-auto-gen-summary
```

(or `cargo install --path .` when building from source.)

It can be use in two ways:

### CLI

```bash
mdbook-auto-gen-summary gen /path/to/your/mdbook/src

# with options...
mdbook-auto-gen-summary gen src -t -T -i index.md,README.md -w draft
```

You can specify the following options (see [configuration](#configuration)):

- `--title` / `-t` sets `first-line-as-link-text` to true
- `--dir-title` / `-T` sets `index-first-line-as-directory-link-text` to true
- `--dir-index-names` / `-i` followed by a comma-separated list sets `directory-index-names`
- `--dir-without-index-behavior` / `-w` followed by a string sets `directory-without-index-behavior`

This will walk your mdbook src dir and generate the book summary in /path/to/your/mdbook/src/SUMMARY.md

### mdbook preprocessor

#### Configuration

Include the following in your `book.toml`:

```toml
[preprocessor.auto-gen-summary]
first-line-as-link-text = true                     # default: false
index-first-line-as-directory-link-text = true     # default: false
directory-index-names = ["index.md", "README.md"]  # default: ["README.md"]
directory-without-index-behavior = "draft"         # default: "ignore"
```

- The first line tells `mdbook` to use this preprocessor.
- `first-line-as-link-text`
  - When `true`, the title of markdown files in `SUMMARY.md` will match the first line of the file's content. The first line must start with `# ` (heading 1) for this to work.
  - When `false`, the title of markdown files match the file's name (ex. `myfile.md`)
- `index-first-line-as-directory-link-text`
  - When `true`, the title of a directory will match the first line of the directory index file's content. The first line must also start with `# ` (heading 1) for this to work.
  - When `false`, the title of directories match the directory name.
- `directory-index-names`
  - A list of file names that can be recognized as directory index files.
- `directory-without-index-behavior`
  - When there is no directory index file in the directory, what should we do? Options:
    - `ignore`: Ignores the directory
    - `draft`: Marks the directory as a draft. The directory text becomes unclickable.
    - `generate-stub-index`: Generates an empty index file for you. The name of the file is the first item listed in the `directory-index-names` option. If `directory-index-names` is not specified, this creates `README.md` files.

#### Additional Optional Configuration

We recommend adding the following option to `book.toml` if using this program as a preprocessor.

```toml
[build]
create-missing = false
```

This option prevents `mdbook` from recreating the markdown files you delete. Note that you will still have to empty `SUMMARY.md` after deleting a file for the book to generate.

---

If you have many markdown files, it may be helpful to make headings collapsible. You can do this by adding the following option.

```toml
[output.html.fold]
enable = true
level = 0
```

#### Running

Running the following commands will generate `src/SUMMARY.md` while building the book.

```bash
mdbook serve
```
Or
```bash
mdbook build
```

## Troubleshooting

### Building the book fails because a Chapter file is not found

If you get an error like this:

```
[...] [ERROR] (mdbook::cmd::watch::poller): failed to load book config: Chapter file not found, [...]

Caused by:
    No such file or directory (os error 2)
```

This may be because you deleted a file. `mdbook` won't continue the build if it cannot find a file listed in `SUMMARY.md`.

The simple solution is to empty the `SUMMARY.md` file or rerun `md-book-auto-gen-summary` manually using the [CLI](#cli).

### I delete a file but the file reappears

#### Solution 1

Stop `mdbook serve`, delete the file, then empty the `SUMMARY.md` file. Then, run `mdbook serve` again.

#### Solution 2

Add the following option to `book.toml`.

```toml
[build]
create-missing = false
```

Then, empty the `SUMMARY.md` file or rerun the [CLI](#cli) before `mdbook` can build the book.

### My folder doesn't appear in the summary

#### Solution 1

Add a `README.md` file to the directory you want to show.

If you have set the option `directory-index-names`, you need to make a file with one of the names specified by `directory-index-names` instead.

#### Solution 2

If you don't want an index file for every directory (i.e, a chapter to open when you click on the folder), you can add the following option to `book.toml`.

```toml
[preprocessor.auto-gen-summary]
directory-without-index-behavior = "draft"
```

If you're using the CLI, you can set this by adding `-w draft` to the end of the command.

#### Solution 3

If you **do** want an index file for every directory, you can create the index files automatically by adding the following option to `book.toml`

```toml
[preprocessor.auto-gen-summary]
directory-without-index-behavior = "generate-stub-index"
```

If you're using the CLI, you can set this by adding `-w generate-stub-index` to the end of the command.
