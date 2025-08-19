# mdbook-auto-gen-summary

A preprocessor and cli tool for mdbook to automatically generate the `SUMMARY.md` file from your existing directory structure.

## Usage

### Install

```bash
cargo install mdbook-auto-gen-summary
```

(or `cargo install --path .` when building from source.)

It can be use in two ways:

### 1. Use as a cli tool.

```bash
mdbook-auto-gen-summary gen /path/to/your/mdbook/src
```

or

```bash
mdbook-auto-gen-summary gen -t /path/to/your/mdbook/src
```

- `-t` sets `first-line-as-link-text` to true (see [configuration](#configuration))

This will walk your mdbook src dir and generate the book summary in /path/to/your/mdbook/src/SUMMARY.md

### 2. Use as mdbook preprocessor.

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
