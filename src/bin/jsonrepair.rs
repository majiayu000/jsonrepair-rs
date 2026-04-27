use std::{
    env,
    ffi::{OsStr, OsString},
    fmt,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process,
};

use jsonrepair_rs::{jsonrepair_reader_to_writer, JsonRepairStreamError};

const HELP: &str = "\
Repair malformed JSON-like text into valid JSON.

Usage:
  jsonrepair [OPTIONS] [INPUT_FILE]

Arguments:
  INPUT_FILE          Read input from this file. Reads stdin when omitted or '-'.

Options:
  -o, --output PATH   Write repaired JSON to PATH instead of stdout.
  -h, --help          Print help.
  -V, --version       Print version.
";

fn main() {
    match run() {
        Ok(()) => {}
        Err(CliError::Usage(message)) => {
            eprintln!("jsonrepair: {message}");
            eprintln!("Try 'jsonrepair --help' for usage.");
            process::exit(2);
        }
        Err(err) => {
            eprintln!("jsonrepair: {err}");
            process::exit(1);
        }
    }
}

fn run() -> Result<(), CliError> {
    let args = Args::parse(env::args_os().skip(1))?;

    if args.help {
        print!("{HELP}");
        return Ok(());
    }

    if args.version {
        println!("jsonrepair {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if let Some(path) = args.input.as_deref().filter(|path| *path != Path::new("-")) {
        let mut input = File::open(path).map_err(|source| CliError::Io {
            action: format!("failed to read {}", path.display()),
            source,
        })?;
        repair_to_output(&mut input, args.output.as_deref())?;
    } else {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        repair_to_output(&mut input, args.output.as_deref())?;
    }

    Ok(())
}

fn repair_to_output(input: &mut dyn Read, output_path: Option<&Path>) -> Result<(), CliError> {
    if let Some(path) = output_path {
        let mut output = File::create(path).map_err(|source| CliError::Io {
            action: format!("failed to write {}", path.display()),
            source,
        })?;
        jsonrepair_reader_to_writer(input, &mut output)?;
    } else {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        jsonrepair_reader_to_writer(input, &mut output)?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct Args {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    help: bool,
    version: bool,
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = Self::default();
        let mut raw_args = raw_args.into_iter();

        while let Some(arg) = raw_args.next() {
            if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
                args.help = true;
            } else if arg == OsStr::new("-V") || arg == OsStr::new("--version") {
                args.version = true;
            } else if arg == OsStr::new("-o") || arg == OsStr::new("--output") {
                let output = raw_args.next().ok_or_else(|| {
                    CliError::Usage("--output requires a path argument".to_string())
                })?;
                args.output = Some(PathBuf::from(output));
            } else if let Some(output) = parse_output_equals(&arg) {
                args.output = Some(PathBuf::from(output));
            } else if looks_like_unknown_option(&arg) {
                return Err(CliError::Usage(format!(
                    "unknown option {}",
                    display_arg(&arg)
                )));
            } else if args.input.replace(PathBuf::from(&arg)).is_some() {
                return Err(CliError::Usage(
                    "expected at most one input file".to_string(),
                ));
            }
        }

        Ok(args)
    }
}

fn parse_output_equals(arg: &OsStr) -> Option<OsString> {
    let arg = arg.to_str()?;
    arg.strip_prefix("--output=").map(OsString::from)
}

fn looks_like_unknown_option(arg: &OsStr) -> bool {
    arg.to_str()
        .is_some_and(|arg| arg.starts_with('-') && arg != "-")
}

fn display_arg(arg: &OsStr) -> String {
    arg.to_string_lossy().into_owned()
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Io { action: String, source: io::Error },
    Repair(JsonRepairStreamError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => f.write_str(message),
            Self::Io { action, source } => write!(f, "{action}: {source}"),
            Self::Repair(err) => err.fmt(f),
        }
    }
}

impl From<JsonRepairStreamError> for CliError {
    fn from(err: JsonRepairStreamError) -> Self {
        Self::Repair(err)
    }
}
