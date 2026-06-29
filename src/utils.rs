use std::io::Write;
use std::sync::{Arc, RwLock};
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum Problem {
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
                write!(
                    formatter,
                    "The file provided does not have a recognized file extension"
                )
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
pub struct WrappedLogger {
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

            if let Some(log_file) = &mut *self.log_file.write().unwrap() {
                log_file
                    .write_all(format!("{}\r\n", record.args()).as_bytes())
                    .ok();
            }
        }
    }

    fn flush(&self) {}
}

pub fn setup_logger() -> std::sync::Arc<std::sync::RwLock<std::option::Option<std::fs::File>>> {
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

    log_file
}
