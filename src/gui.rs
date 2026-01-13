use std::fmt::Display;

pub fn run() -> Result<(), GUIError> {
    println!("GUI Launched");
    Ok(())
}
#[derive(Debug)]
pub enum GUIError {}
impl Display for GUIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
