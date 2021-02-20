use druid::{WindowDesc, AppLauncher};
use crate::ui::main_ui;
use crate::data::GameData;

mod data;
mod ui;

fn main() {
    let window = WindowDesc::new(main_ui)
        .with_min_size((560.0, 680.0))
        .resizable(true)
        .title("Tick Tack Toe Ultimate");

    AppLauncher::with_window(window)
        .launch(GameData::new())
        .expect("launch failed!");
}
