#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use rbxlx_to_rojo::{
    filesystem::FileSystem,
    process_instructions,
    utils::{Problem, setup_logger},
};
use std::{
    borrow::Cow,
    fs,
    io::BufReader,
};

fn routine() -> Result<(), Problem> {
    let log_file = setup_logger();
    info!("rbxlx-to-rojo {}", env!("CARGO_PKG_VERSION"));

    info!("Select a place file.");
    let file_path: std::path::PathBuf = {
        let path = std::env::args().nth(1);
        Ok(match path {
            Some(text) => text.into(),
            None => {
                #[cfg(feature = "file_picker")]
                match rfd::FileDialog::new()
                    .add_filter("rbx", &["rbxl", "rbxlx", "rbxm", "rbxmx"])
                    .pick_file()
                {
                    Some(p) => p,
                    None => Err(Problem::FileDialogueError("File Error".into()))?,
                }
                #[cfg(not(feature = "file_picker"))]
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
                #[cfg(feature = "file_picker")]
                match rfd::FileDialog::new().pick_folder() {
                    Some(p) => p,
                    None => Err(Problem::FileDialogueError("Folder Error".into()))?,
                }
                #[cfg(not(feature = "file_picker"))]
                Err(Problem::InvalidFile)?
            }
        })
    }?;

    let mut filesystem = FileSystem::from_root(root.join(file_path.file_stem().unwrap()));

    log_file.write().unwrap().replace(
        fs::File::create(root.join("rbxlx-to-rojo.log"))
            .map_err(|error| Problem::IoError("couldn't create log file", error))?,
    );

    process_instructions(&tree, &mut filesystem);
    Ok(())
}

fn main() {
    #[cfg(feature = "gui")]
    {
        use std::env;

        let args = env::args().nth(1);
        if args.is_none() {
            if let Err(error) = rbxlx_to_rojo::gui::run() {
                eprintln!("An error occurred while using rbxlx-to-rojo.");
                eprintln!("{}", error);
            }
            return;
        }
    }
    if let Err(error) = routine() {
        eprintln!("An error occurred while using rbxlx-to-rojo.");
        eprintln!("{}", error);
    }
}
