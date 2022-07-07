use std::{
    fs::File,
    io::{BufRead, BufReader, Error as IoError, Lines, Write},
};

const USAGE: &str = "USAGE: doc2book CRATE_DIR OUT_FILE_PATH";

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    CrateSourceDirNotFound(String),
    LibRsNotFound(String),
    ModuleSourceNotFound(String),
    Io(IoError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

// source reading interface
trait ReadSource {
    type Src: BufRead;

    fn lib_rs_path(&self) -> &str;

    fn module_path_from_name(&self, module: &str) -> String;

    fn lines_from_path(&self, src_path: &str) -> Result<Lines<Self::Src>>;
}

// source reading implementor for crates
struct CrateSource {
    crate_src_dir: String,
    lib_rs_path: String,
}

impl CrateSource {
    // ensure the crate is valid,
    // i.e. it's src directory exists and contains a mod file
    fn new(crate_dir: &str) -> Result<CrateSource> {
        let crate_src_dir = format!("{}/src", crate_dir);
        let lib_rs_path = format!("{}/mod", crate_src_dir);

        if std::fs::metadata(&crate_src_dir).is_err() {
            return Err(Error::CrateSourceDirNotFound(crate_src_dir));
        }

        if std::fs::metadata(&lib_rs_path).is_err() {
            return Err(Error::LibRsNotFound(lib_rs_path));
        }

        Ok(CrateSource {
            crate_src_dir,
            lib_rs_path,
        })
    }
}

impl ReadSource for CrateSource {
    type Src = BufReader<File>;

    fn lib_rs_path(&self) -> &str {
        &self.lib_rs_path
    }

    fn module_path_from_name(&self, module: &str) -> String {
        format!("{}/{}.rs", self.crate_src_dir, module)
    }

    fn lines_from_path(&self, src_path: &str) -> Result<Lines<Self::Src>> {
        if std::fs::metadata(src_path).is_err() {
            return Err(Error::ModuleSourceNotFound(src_path.to_string()));
        }

        let file = File::open(src_path)?;
        let lines = BufReader::new(file).lines();

        Ok(lines)
    }
}

struct Output {
    output: Vec<u8>,
}

impl Output {
    fn new() -> Output {
        // allocate a decent chunk of memory at the start
        let output = Vec::with_capacity(1_000_000_000);
        Output { output }
    }

    fn push_line(&mut self, line: &str) {
        writeln!(&mut self.output, "{}", line).unwrap();
    }

    fn write_into(&self, w: &mut dyn Write) -> Result<()> {
        w.write_all(&self.output)?;
        Ok(())
    }
}

fn parse_args() -> Option<(String, String)> {
    let mut args = std::env::args().skip(1);
    let crate_dir = args.next()?;
    let out_path = args.next()?;
    Some((crate_dir, out_path))
}

fn find_doc_comment(line: &str) -> Option<&str> {
    line.strip_prefix("//! ")
        .or_else(|| line.strip_prefix("//# "))
        .or_else(|| line.strip_prefix("/// "))
}

// scrape from code between pragmas and each type of doc comment
fn process_module<I>(input: &I, output: &mut Output, module_name: &str) -> Result<()>
where
    I: ReadSource,
{
    let module_path = input.module_path_from_name(module_name);
    eprintln!("Processing module file: {}", module_path);

    let mut code_transcribe_enabled = false;

    for line in input.lines_from_path(&module_path)? {
        let line = line?;

        if line.starts_with("// book_include_code") {
            output.push_line("```rust");
            code_transcribe_enabled = true;
            continue;
        }

        if line.starts_with("// end_book_include_code") {
            output.push_line("```");
            code_transcribe_enabled = false;
            continue;
        }

        if code_transcribe_enabled {
            output.push_line(&line);
            continue;
        }

        if let Some(line) = find_doc_comment(&line) {
            output.push_line(line);
        }
    }

    Ok(())
}

// start with the crate root, mod
// scrape code comments and with each public module inserted in the order they appear
fn process_crate<I>(input: I, output: &mut Output) -> Result<()>
where
    I: ReadSource,
{
    let lib_rs_path = input.lib_rs_path();
    eprintln!("Processing mod file: {}", lib_rs_path);

    for line in input.lines_from_path(lib_rs_path)? {
        let line = line?;

        if let Some(doc) = find_doc_comment(&line) {
            output.push_line(doc);
            continue;
        }

        if line.starts_with("pub mod ") && line.ends_with(';') {
            let module = line
                .strip_prefix("pub mod ")
                .unwrap()
                .strip_suffix(';')
                .unwrap();

            process_module(&input, output, module)?;
        }
    }

    Ok(())
}

fn main() {
    let (crate_root_dir, out_path) = match parse_args() {
        Some(args) => args,
        _ => return eprintln!("{}", USAGE),
    };

    let input = match CrateSource::new(&crate_root_dir) {
        Ok(input) => input,
        Err(Error::CrateSourceDirNotFound(path)) => {
            eprintln!(
                "{} does not exist. Make sure the specified directory is a Rust project.",
                path
            );
            return eprintln!("{}", USAGE);
        }
        Err(Error::LibRsNotFound(path)) => {
            eprintln!("{} not does not exist. The crate must be a library.", path);
            return eprintln!("{}", USAGE);
        }
        Err(err) => panic!("unexpected err: {:?}", err),
    };

    let mut output = Output::new();

    let result = process_crate(input, &mut output).and_then(|_| {
        let mut out_file = File::create(out_path)?;
        output.write_into(&mut out_file)
    });

    if let Err(err) = result {
        match err {
            Error::ModuleSourceNotFound(path) => {
                eprintln!(
                    "Processing module file {} failed: file does not exist",
                    path
                )
            }
            Error::Io(err) => eprintln!("Unexpected I/O Error: {}", err),
            err => panic!("unexpected err: {:?}", err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const LIB_RS: &str = r#"//! # Main Title
//! Some text
//! 

pub mod helper_mod;

//# ## Common Types
//# Some more text
//# 

pub mod common_types;
"#;

    const COMMON_TYPES_RS: &str = r#"use some_dep::module;    
use some_other_dep::module;

//# ### `Contract`
/// Represents another contract on the network
/// 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// book_include_code
pub struct Contract {
    /// The address of the contract
    pub address: Addr,
    /// The hex encoded hash of the contract code
    pub code_hash: String,
}
// end_book_include_code"#;

    const EXPECTED_OUTPUT: &str = r#"# Main Title
Some text

## Common Types
Some more text

### `Contract`
Represents another contract on the network

```rust
pub struct Contract {
    /// The address of the contract
    pub address: Addr,
    /// The hex encoded hash of the contract code
    pub code_hash: String,
}
```
"#;

    struct TestInput;

    impl ReadSource for TestInput {
        type Src = BufReader<&'static [u8]>;

        fn lib_rs_path(&self) -> &str {
            "src/mod"
        }

        fn module_path_from_name(&self, module: &str) -> String {
            format!("src/{}.rs", module)
        }

        fn lines_from_path(&self, src_path: &str) -> Result<Lines<Self::Src>> {
            let s: &'static str = match src_path {
                "src/mod" => LIB_RS,
                "src/common_types.rs" => COMMON_TYPES_RS,
                "src/helper_mod.rs" => "",
                _ => panic!("Unexpected path: {}", src_path),
            };

            Ok(BufReader::new(s.as_bytes()).lines())
        }
    }

    #[test]
    fn it_works() {
        let mut output = Output::new();

        process_crate(TestInput, &mut output).unwrap();

        let mut actual = Vec::new();

        output.write_into(&mut actual).unwrap();

        let actual = String::from_utf8(actual).unwrap();

        assert_eq!(actual, EXPECTED_OUTPUT.to_owned())
    }
}
