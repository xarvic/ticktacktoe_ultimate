use druid::{WindowDesc, AppLauncher};
use crate::ui::main_ui;
use crate::data::GameData;

mod data;
mod ui;

fn main() {
    let window = WindowDesc::new(main_ui)
        .window_size((560.0, 660.0))
        .resizable(false)
        .title("Tick Tack Toe Ultimate");

    AppLauncher::with_window(window)
        .launch(GameData::new())
        .expect("launch failed!");
}
