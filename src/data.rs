use druid::{Data, Lens};
use std::ops::{Index, Deref};
use std::sync::Arc;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub enum Mark {
    Cross,
    Circle,
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
    slots: [[S; 3]; 3],
    finished: Option<Mark>,
    has_free: bool,
}

type Field = Grid<Option<Mark>>;

type LargeField = Grid<Field>;

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
        let has_free = self.slots.iter().flat_map(|x|x).any(|slot|slot.has_free());
        self.has_free = has_free;
    }

    pub fn set(&mut self, x: usize, y: usize, mark: S) {
        self.slots[x][y] = mark;
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
        let row = [S::empty(), S::empty(), S::empty()];
        Grid {
            slots: [row.clone(), row.clone(), row.clone()],
            finished: None,
            has_free: true,
        }
    }
}

impl<S: Slot + Clone + Eq> Index<(usize, usize)> for Grid<S> {
    type Output = S;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.slots[index.0][index.1]
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Data)]
pub struct FieldMeta {
    field: Field,
    next_turn: Mark,
    written: Option<(usize, usize)>,
    active: bool,
}

impl FieldMeta {
    pub fn from_data(game_data: &GameData, position: (usize, usize)) -> Self {
        FieldMeta {
            field: game_data.game[position].clone(),
            next_turn: game_data.next_turn,
            active: (game_data.next_field == Some(position) || game_data.next_field == None) &&
                game_data.game[position].has_free() &&
                game_data.game.belongs_to().is_none(),
            written: None,
        }
    }
    pub fn write_back(mut self, game_data: &mut GameData, field_position: (usize, usize)) {
        if let Some((x, y)) = self.written {
            self.field.set(x, y, Some(self.next_turn));
            game_data.game.set(field_position.0, field_position.1, self.field);

            game_data.next_turn = if self.next_turn == Mark::Cross {
                Mark::Circle
            } else {
                Mark::Cross
            };
            game_data.next_field = if game_data.game[(x, y)].has_free() {
                Some((x, y))
            } else {
                None
            };
        }
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn set(&mut self, position: (usize, usize)) {
        if self.active {
        self.written = Some(position);
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

#[derive(Clone, Data, Lens)]
pub struct GameData {
    pub game: LargeField,
    pub next_turn: Mark,
    pub next_field: Option<(usize, usize)>,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            game: LargeField::empty(),
            next_turn: Mark::Cross,
            next_field: None,
        }
    }
}