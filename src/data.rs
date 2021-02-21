use druid::{Data, Lens, ExtEventSink};
use std::ops::{Index, Deref};
use crate::data::Mark::{Cross, Circle};
use crate::ai::best_move;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub enum Mark {
    Cross,
    Circle,
}

impl Mark {
    pub fn other(&self) -> Mark {
        match self {
            Mark::Cross => Circle,
            Mark::Circle => Cross,
        }
    }
}

impl Slot for Option<Mark> {
    fn belongs_to(&self) -> Option<Mark> {
        *self
    }

    fn has_free(&self) -> bool {
        self.is_none()
    }

    fn empty() -> Self {
        None
    }
}

pub trait Slot {
    fn belongs_to(&self) -> Option<Mark>;
    fn has_free(&self) -> bool;
    fn empty() -> Self;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub struct Grid<S: Slot + Clone + Eq> {
    slots: [S; 9],
    finished: Option<Mark>,
    has_free: bool,
}

pub type Field = Grid<Option<Mark>>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub struct FieldPosition(usize);

impl From<(usize, usize)> for FieldPosition {
    fn from(pos: (usize, usize)) -> Self {
        let val = pos.0 + pos.1 * 3;

        assert!(val < 9);

        Self(val)
    }
}

impl FieldPosition {
    fn index(self) -> usize {
        self.0
    }
    pub fn x(self) -> usize {
        self.0 % 3
    }
    pub fn y(self) -> usize {
        self.0 / 3
    }
    pub fn all() -> impl Iterator<Item=FieldPosition> + Clone {
        (0..9).map(|x|Self(x))
    }
}

pub type LargeField = Grid<Field>;

#[derive(Copy, Clone, Eq, PartialEq)]
struct LargeFieldPosition(usize);

impl<S: Slot + Eq + Clone> Grid<S> {
    fn row_finished(&self, row: usize) -> Option<Mark> {
        if self[(0, row)].belongs_to() == self[(1, row)].belongs_to() && self[(1, row)].belongs_to() == self[(2, row)].belongs_to() {
            self[(0, row)].belongs_to()
        } else {
            None
        }
    }
    fn column_finished(&self, column: usize) -> Option<Mark> {
        if self[(column, 0)].belongs_to() == self[(column, 1)].belongs_to() && self[(column, 1)].belongs_to() == self[(column, 2)].belongs_to() {
            self[(column, 0)].belongs_to()
        } else {
            None
        }
    }
    fn diagonal_finished(&self) -> Option<Mark> {
        if (self[(0, 0)].belongs_to() == self[(1, 1)].belongs_to() && self[(1, 1)].belongs_to() == self[(2, 2)].belongs_to()) ||
           (self[(0, 2)].belongs_to() == self[(1, 1)].belongs_to() && self[(1, 1)].belongs_to() == self[(2, 0)].belongs_to()) {
            self[(1, 1)].belongs_to()
        } else {
            None
        }
    }

    fn calc_finished(&mut self) {
        let finished = self.diagonal_finished()
            .or(self.row_finished(0))
            .or(self.row_finished(1))
            .or(self.row_finished(2))
            .or(self.column_finished(0))
            .or(self.column_finished(1))
            .or(self.column_finished(2));
        self.finished = finished;
        let has_free = self.slots.iter().any(|slot|slot.has_free());
        self.has_free = has_free;
    }

    pub fn set(&mut self, pos: impl Into<FieldPosition>, mark: S) {
        self.slots[pos.into().index()] = mark;
        self.calc_finished();
    }
}

impl<S: Slot + Clone + Eq> Slot for Grid<S> {
    fn belongs_to(&self) -> Option<Mark> {
        self.finished
    }

    fn has_free(&self) -> bool {
        self.has_free && self.belongs_to().is_none()
    }

    fn empty() -> Self {
        Grid {
            slots: [
                S::empty(), S::empty(), S::empty(),
                S::empty(), S::empty(), S::empty(),
                S::empty(), S::empty(), S::empty(),
            ],
            finished: None,
            has_free: true,
        }
    }
}

impl<S: Slot + Clone + Eq, P: Into<FieldPosition>> Index<P> for Grid<S> {
    type Output = S;

    fn index(&self, index: P) -> &Self::Output {
        &self.slots[index.into().index()]
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Data)]
pub struct FieldMeta {
    field: Field,
    next_turn: Mark,
    written: Option<FieldPosition>,
    active: bool,
}

impl FieldMeta {
    pub fn from_data(game_data: &GameData, position: impl Into<FieldPosition> + Clone) -> Self {
        FieldMeta {
            field: game_data.game[position.clone()].clone(),
            next_turn: game_data.next_turn,
            active: (game_data.next_field == Some(position.clone().into()) || game_data.next_field == None) &&
                game_data.game[position.into()].has_free() &&
                game_data.game.belongs_to().is_none() &&
                game_data.my_turn(),
            written: None,
        }
    }
    pub fn write_back(mut self, game_data: &mut GameData, field_position: impl Into<FieldPosition>) {
        if let Some(position) = self.written {
            self.field.set(position, Some(self.next_turn));
            game_data.game.set(field_position, self.field);

            game_data.next_turn = game_data.next_turn.other();
            game_data.next_field = if game_data.game[position].has_free() {
                Some(position)
            } else {
                None
            };
        }
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn set(&mut self, position: impl Into<FieldPosition>) {
        if self.active {
        self.written = Some(position.into());
        }
    }
    pub fn next_turn(&self) -> Mark {
        self.next_turn
    }
}

impl Deref for FieldMeta {
    type Target = Field;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

#[derive(Copy, Clone, Data)]
pub enum Opponent {
    Ai {level: u64},
}

#[derive(Clone, Data, Lens)]
pub struct GameData {
    pub game: LargeField,
    pub next_turn: Mark,
    pub next_field: Option<FieldPosition>,
    pub opponent: Option<(Opponent, Mark)>,
}

impl GameData {
    pub fn local() -> Self {
        Self {
            game: LargeField::empty(),
            next_turn: Mark::Cross,
            next_field: None,
            opponent: None,
        }
    }

    pub fn ai(level: u64) -> Self {
        Self {
            game: LargeField::empty(),
            next_turn: Mark::Cross,
            next_field: None,
            opponent: Some((Opponent::Ai {level}, Mark::Circle)),
        }
    }

    pub fn handle_opponent(&self, sink: ExtEventSink) {
        if let Some((Opponent::Ai {level}, mark)) = self.opponent.as_ref() {
            if *mark == self.next_turn {
                best_move(self.game.clone(), *mark, self.next_field, *level, sink);
            }
        }
    }

    pub fn my_turn(&self) -> bool {
        self.opponent.as_ref().map(|op|op.1) != Some(self.next_turn)
    }
}