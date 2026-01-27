use std::{
    borrow::Cow,
    fmt::Display,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use eframe::{
    App,
    egui::{CentralPanel, Color32, Context, Ui, ViewportBuilder},
};

use crate::{filesystem::FileSystem, process_instructions};

pub fn run() -> Result<(), GUIError> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([380.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rbxlx to Rojo",
        options,
        Box::new(|_| Ok(Box::<MainApp>::default())),
    )
    .map_err(GUIError::Gui)?;
    Ok(())
}

#[derive(Debug, Default)]
struct PickInput {
    rx: Option<mpsc::Receiver<PathBuf>>,
    input: String,
}

impl PickInput {
    fn show(&mut self, _ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.input);
            let button = ui.button("Pick File");
            if button.clicked() && self.rx.is_none() {
                let (tx, rx) = mpsc::channel();
                self.rx = Some(rx);
                let prev = self.input.clone();
                thread::spawn(move || {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("rbx", &["rbxl", "rbxlx", "rbxm", "rbxmx"])
                        .set_file_name(prev)
                        .pick_file()
                    {
                        let _ = tx.send(p);
                    }
                });
            }
            if let Some(rx) = &mut self.rx
                && let Ok(p) = rx.try_recv()
            {
                self.rx = None;
                self.input = p.to_string_lossy().to_string();
            }
        });
    }
}

#[derive(Debug, Default)]
struct PickOutput {
    output: String,
    rx: Option<mpsc::Receiver<PathBuf>>,
}
impl PickOutput {
    fn show(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.output);
            let button = ui.button("Pick File");
            if button.clicked() && self.rx.is_none() {
                let (tx, rx) = mpsc::channel();
                self.rx = Some(rx);
                let prev = self.output.clone();
                thread::spawn(move || {
                    if let Some(p) = rfd::FileDialog::new().set_directory(&prev).pick_folder() {
                        let _ = tx.send(p);
                    };
                });
            }
            if let Some(rx) = &mut self.rx
                && let Ok(p) = rx.try_recv()
            {
                self.output = p.to_string_lossy().to_string();
            }
        });
    }
}

#[derive(Debug, Default)]
struct MainApp {
    output: PickOutput,
    error_msg: Option<String>,
    success: bool,
    input: PickInput,
}

impl MainApp {
    fn process(&mut self) -> Result<(), String> {
        if self.input.input.is_empty() {
            return Err("No Input File Provided".to_string());
        } else if self.output.output.is_empty() {
            return Err("No Output File Provided".to_string());
        };
        let input_path = Path::new(&self.input.input).to_path_buf();
        let output_path = Path::new(&self.output.output).to_path_buf();

        let file_source = BufReader::new(
            fs::File::open(&input_path).map_err(|e| format!("Input File Error: {e}"))?,
        );
        let tree = match input_path
            .extension()
            .map(|extension| extension.to_string_lossy())
        {
            Some(Cow::Borrowed("rbxmx")) | Some(Cow::Borrowed("rbxlx")) => {
                rbx_xml::from_reader_default(file_source)
                    .map_err(|e| format!("Input File Decode Error: {e}"))
            }
            Some(Cow::Borrowed("rbxm")) | Some(Cow::Borrowed("rbxl")) => {
                rbx_binary::from_reader(file_source)
                    .map_err(|e| format!("Input File Binary Decode Error: {e}"))
            }
            _ => Err("Invalid Input File".to_string()),
        }?;
        let mut filesystem =
            FileSystem::from_root(output_path.join(input_path.file_stem().unwrap()));
        process_instructions(&tree, &mut filesystem);
        Ok(())
    }
}

impl App for MainApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("rbxlx to Rojo");
            ui.add_space(5.0);
            ui.label("rbxlx File");
            self.input.show(ctx, ui);
            ui.add_space(10.0);

            ui.label("Directory Output");
            self.output.show(ctx, ui);
            ui.add_space(2.0);
            ui.label("This will create or use an existing directory in that output directory named by the rbxlx file");
            ui.add_space(10.0);

            let enter = ui.button("Enter");
            if enter.clicked() {
                if let Err(e) = self.process() {
                    self.error_msg = Some(e);
                } else {
                    self.error_msg = None;
                    self.success = true;
                };
            }
            ui.add_space(3.0);
            if let Some(err) = &self.error_msg {
                ui.colored_label(Color32::RED, err.to_string());
            } else if self.success {
                ui.colored_label(Color32::GREEN, "Success");
            }
        });
    }
}

#[derive(Debug)]
pub enum GUIError {
    Gui(eframe::Error),
}
impl Display for GUIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GUIError::Gui(error) => write!(f, "Gui Error: {}", error),
        }
    }
}
