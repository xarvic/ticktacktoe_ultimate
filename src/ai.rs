use crate::data::{LargeField, Mark, FieldPosition, Slot};
use itertools::Itertools;
use std::cmp::Ordering;
use std::time::{Instant, Duration};
use std::thread::{sleep, spawn};
use druid::{ExtEventSink, Selector, Target};
use std::f32::{INFINITY, NEG_INFINITY};

fn game_state(field: LargeField, mark: Mark) -> f32 {
    let won_field = FieldPosition::all()
        .map(|pos|field[pos].belongs_to().map(|m|m==mark))
        .filter_map(|x|x)
        .fold(0.0, |state, x|if x {state + 1.0} else {state - 1.0});

    let tactic_position: f32 = FieldPosition::all()
        .filter(|&pos|{
            field[pos].has_free()
        })
        .cartesian_product(FieldPosition::all())
        .filter(|(outer, inner)|field[*outer][*inner].has_free())
        .map(|(outer, inner_pos)|{
            let mut inner = field[outer];
            inner.set(inner_pos, Some(mark));
            let x = if inner.belongs_to().is_some() {1.0} else {0.0};
            inner.set(inner_pos, Some(mark.other()));
            x + if inner.belongs_to().is_some() {-1.0} else {0.0}
        })
        .sum();

    let won = match field.belongs_to().map(|m|m==mark) {
        Some(true) => INFINITY,
        Some(false) => NEG_INFINITY,
        None => 0.0,
    };

    won + won_field + tactic_position * 0.3
}

pub static MAKE_MOVE: Selector<(FieldPosition, FieldPosition)> = Selector::new("de.ticktacktoe_ultimate.make_move");

pub fn best_move(field: LargeField, mark: Mark, next_field: Option<FieldPosition>, ahead: u64, sink: ExtEventSink) {
    spawn(move ||{
        let start = Instant::now();

        let ret = calc_move(field, mark, next_field, ahead * 2);

        if let Some(duration) = Duration::from_millis(800).checked_sub(start.elapsed()) {
            sleep(duration);
        }
        sink.submit_command(MAKE_MOVE, (ret.0, ret.1), Target::Global).unwrap();
    });
}

fn calc_move(field: LargeField, mark: Mark, next_field: Option<FieldPosition>, steps: u64) -> (FieldPosition, FieldPosition, f32) {
        let iter = FieldPosition::all()
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
                    calc_move(new_field, mark.other(), next_pos, steps - 1).2
                } else {
                    game_state(field, mark)
                };
                (outer, inner, r)
            });
    if steps % 2 == 0 {
        iter.max_by(|v0, v1| v0.2.partial_cmp(&v1.2).unwrap_or(Ordering::Equal))
            .unwrap()
    } else {
        iter.min_by(|v0, v1| v0.2.partial_cmp(&v1.2).unwrap_or(Ordering::Equal))
            .unwrap()
    }
}