use crate::data::{LargeField, Mark, FieldPosition, Slot};
use itertools::Itertools;
use std::cmp::Ordering;
use std::time::{Instant, Duration};
use std::thread::{sleep, spawn};
use druid::{ExtEventSink, Selector, Target};

#[derive(Copy, Clone, Eq, PartialEq)]
struct Metrik {
}

impl Metrik {
    pub fn comp(&self, other: &Self) -> Ordering {
        Ordering::Equal
    }
}

fn game_state(_field: LargeField, _mark: Mark) -> Metrik {
    Metrik {}
}

pub static MAKE_MOVE: Selector<(FieldPosition, FieldPosition)> = Selector::new("de.ticktacktoe_ultimate.make_move");

pub fn best_move(field: LargeField, mark: Mark, next_field: Option<FieldPosition>, ahead: u64, sink: ExtEventSink) {
    println!("make move!");
    spawn(move ||{
        let start = Instant::now();

        let ret = calc_move(field, mark, next_field, ahead * 2);

        if let Some(duration) = Duration::from_millis(800).checked_sub(start.elapsed()) {
            sleep(duration);
        }
        sink.submit_command(MAKE_MOVE, (ret.0, ret.1), Target::Global);
        println!("finished!");
    });
}

fn calc_move(field: LargeField, mark: Mark, next_field: Option<FieldPosition>, steps: u64) -> (FieldPosition, FieldPosition, Metrik) {
        FieldPosition::all()
            .filter(|&pos|{
                next_field.is_none() || next_field == Some(pos)
            })
            .cartesian_product(FieldPosition::all())
            .filter(|(outer, inner)|field[*outer][*inner].has_free())
            .map(|(outer, inner)|{
                let mut new_field = field.clone();
                let mut new_inner = new_field[outer].clone();
                new_inner.set(inner, Some(mark));
                new_field.set(outer, new_inner);

                let next_pos = if new_field[inner].has_free() {
                    Some(inner)
                } else {
                    None
                };

                let r = if steps > 0 && field.has_free() {
                    calc_move(new_field, mark.other(), next_field, steps - 1).2
                } else {
                    game_state(field, mark)
                };
                (outer, inner, r)
            })
            .max_by(|v0, v1|v0.2.comp(&v1.2))
            .unwrap()
}