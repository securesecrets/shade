# `doc2book`
A simple tool that scrapes rustdoc comments, `doc2book`-only comments and specified Rust code from a crate's source files 
and outputs them as a single file (e.g. a Markdown book chapter).

## Usage
The usage is simple, specify the crate and the output file path
```
USAGE: doc2book CRATE_DIR OUT_FILE_PATH
```

Example from Shade workspace root directory:
```
cargo r -p doc2book --release -- packages/shade_protocol doc/book/src/smart_contracts.md
```

### Comments
The following comments are scraped:
- `//! `: [rustdoc][1] crate-level (inner) doc comments    
- `/// `: [rustdoc][1] code-level (outer) doc comments    
- `//# `: `doc2book` comments, these will be ignored by rustdoc but picked up by `doc2book`

[1]: https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html

### Code
The comments `// book_include_code` and `// end_book_include_code` mark a section of code to be copied verbatim and place in a Rust code block (Markdown syntax).
This code:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// book_include_code
pub struct Contract {
    /// The address of the contract
    pub address: Addr,
    /// The hex encoded hash of the contract code
    pub code_hash: String,
}
// end_book_include_code
```

Would result in this Markdown section:

~~~
```rust
pub struct Contract {
    /// The address of the contract
    pub address: Addr,
    /// The hex encoded hash of the contract code
    pub code_hash: String,
}
```
~~~

## Limitations
As the tool is currently very simple it has a few limitations:
- The crate structure is expected to be one file per module at the same directory level as the lib.rs file.
- The tool outputs everything as a single file, it could be made to create a 'sub-chapter' file for each module.
- Due to it creating a single file, the ordering of comments, modules declarations and the code is the order it will appear output file.
- Adding whitespace between scraped sections needs to be explicit, i.e. an empty comment line needs to be inserted where you want a blank line `//! `.
