use druid::{WindowDesc, AppLauncher, AppDelegate, Handled, Env, Command, Target, DelegateCtx};
use crate::ui::main_ui;
use crate::data::{GameData, Slot};
use crate::ai::MAKE_MOVE;

pub mod data;
mod ui;
mod ai;

struct MyDelegate;

impl AppDelegate<GameData> for MyDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut GameData,
        _env: &Env,
    ) -> Handled {
        if let Some((outer_pos, inner_pos)) = cmd.get(MAKE_MOVE) {
            let next = data.next_turn;

            let mut inner = data.game[*outer_pos];
            inner.set(*inner_pos, Some(next));
            data.game.set(*outer_pos, inner);
            data.next_turn = data.next_turn.other();
            let next_pos = if data.game[*inner_pos].has_free() {
                Some(*inner_pos)
            } else {
                None
            };
            data.next_field = next_pos;

            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn main() {
    let window = WindowDesc::new(main_ui)
        .with_min_size((560.0, 680.0))
        .resizable(true)
        .title("Tick Tack Toe Ultimate");

    AppLauncher::with_window(window)
        .delegate(MyDelegate)
        .launch(GameData::local())
        .expect("launch failed!");
}
