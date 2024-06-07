use log::info;
use rbxlx_to_rojo::{filesystem::FileSystem, process_instructions};
use std::{
    borrow::Cow,
    fmt, fs,
    io::{self, BufReader, Write},
    sync::{Arc, RwLock},
};

#[derive(Debug)]
enum Problem {
    BinaryDecodeError(rbx_binary::DecodeError),
    InvalidFile,
    IoError(&'static str, io::Error),
    FileDialogueCancel,
    FileDialogueError(String),
    XMLDecodeError(rbx_xml::DecodeError),
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Problem::BinaryDecodeError(error) => write!(
                formatter,
                "While attempting to decode the place file, at {} rbx_binary didn't know what to do",
                error,
            ),

            Problem::InvalidFile => {
                write!(formatter, "The file provided does not have a recognized file extension")
            }

            Problem::IoError(doing_what, error) => {
                write!(formatter, "While attempting to {}, {}", doing_what, error)
            }
            Problem::FileDialogueCancel => write!(formatter, "Didn't choose a file."),

            Problem::FileDialogueError(error) => write!(
                formatter,
                "Something went wrong when choosing a file: {}",
                error,
            ),

            Problem::XMLDecodeError(error) => write!(
                formatter,
                "While attempting to decode the place file, at {} rbx_xml didn't know what to do",
                error,
            ),
        }
    }
}

struct WrappedLogger {
    log: env_logger::Logger,
    log_file: Arc<RwLock<Option<fs::File>>>,
}

impl log::Log for WrappedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.log.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.log.log(record);

            if let Some(ref mut log_file) = &mut *self.log_file.write().unwrap() {
                log_file
                    .write(format!("{}\r\n", record.args()).as_bytes())
                    .ok();
            }
        }
    }

    fn flush(&self) {}
}

fn routine() -> Result<(), Problem> {
    let env_logger = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .build();

    let log_file = Arc::new(RwLock::new(None));
    let logger = WrappedLogger {
        log: env_logger,
        log_file: Arc::clone(&log_file),
    };

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("rbxlx-to-rojo {}", env!("CARGO_PKG_VERSION"));

    info!("Select a place file.");
    let file_path: std::path::PathBuf = {
        let path = std::env::args().nth(1);
        Ok(match path {
            Some(text) => text.into(),
            None => {
                #[cfg(feature = "gui")]
                match rfd::FileDialog::new()
                    .add_filter("rbx", &["rbxl", "rbxlx", "rbxm", "rbxmx"])
                    .pick_file()
                {
                    Some(p) => p,
                    None => Err(Problem::FileDialogueError("File Error".into()))?,
                }
                #[cfg(not(feature = "gui"))]
                Err(Problem::InvalidFile)?
            }
        })
    }?;

    info!("Opening place file");
    let file_source = BufReader::new(
        fs::File::open(&file_path)
            .map_err(|error| Problem::IoError("read the place file", error))?,
    );
    info!("Decoding place file, this is the longest part...");

    let tree = match file_path
        .extension()
        .map(|extension| extension.to_string_lossy())
    {
        Some(Cow::Borrowed("rbxmx")) | Some(Cow::Borrowed("rbxlx")) => {
            rbx_xml::from_reader_default(file_source).map_err(Problem::XMLDecodeError)
        }
        Some(Cow::Borrowed("rbxm")) | Some(Cow::Borrowed("rbxl")) => {
            rbx_binary::from_reader(file_source).map_err(Problem::BinaryDecodeError)
        }
        _ => Err(Problem::InvalidFile),
    }?;

    info!("Select the path to put your Rojo project in.");
    let root: std::path::PathBuf = {
        let path = std::env::args().nth(2);
        Ok(match path {
            Some(text) => text.into(),
            None => {
                #[cfg(feature = "gui")]
                match rfd::FileDialog::new().pick_folder() {
                    Some(p) => p,
                    None => Err(Problem::FileDialogueError("Folder Error".into()))?,
                }
                #[cfg(not(feature = "gui"))]
                Err(Problem::InvalidFile)?
            }
        })
    }?;

    let mut filesystem = FileSystem::from_root(root.join(file_path.file_stem().unwrap()));

    log_file.write().unwrap().replace(
        fs::File::create(root.join("rbxlx-to-rojo.log"))
            .map_err(|error| Problem::IoError("couldn't create log file", error))?,
    );

    info!("Starting processing, please wait a bit...");
    process_instructions(&tree, &mut filesystem);
    info!("Done! Check rbxlx-to-rojo.log for a full log.");
    Ok(())
}

fn main() {
    if let Err(error) = routine() {
        eprintln!("An error occurred while using rbxlx-to-rojo.");
        eprintln!("{}", error);
    }
}
