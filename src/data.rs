use druid::{Data, Lens};
use std::ops::{Index, Deref};
use std::sync::Arc;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub enum Mark {
    Cross,
    Circle,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub struct Field {
    slots: [[Option<Mark>; 3]; 3],
    finished: Option<Mark>,
}

impl Field {
    pub fn new() -> Self {
        Field {
            slots: [[None; 3]; 3],
            finished: None,
        }
    }
    fn row_finished(&self, row: usize) -> Option<Mark> {
        if self[(0, row)] == self[(1, row)] && self[(1, row)] == self[(2, row)] {
            self[(0, row)]
        } else {
            None
        }
    }
    fn column_finished(&self, column: usize) -> Option<Mark> {
        if self[(column, 0)] == self[(column, 1)] && self[(column, 1)] == self[(column, 2)] {
            self[(column, 0)]
        } else {
            None
        }
    }
    fn diagonal_finished(&self) -> Option<Mark> {
        if (self[(0, 0)] == self[(1, 1)] && self[(1, 1)] == self[(2, 2)]) ||
           (self[(0, 2)] == self[(1, 1)] && self[(1, 1)] == self[(2, 0)]) {
            self[(1, 1)]
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
    }

    pub fn set(&mut self, x: usize, y: usize, mark: Mark) {
        self.slots[x][y] = Some(mark);
        self.calc_finished();
    }

    pub fn finished(&self) -> Option<Mark> {
        self.finished
    }

    pub fn playable(&self) -> bool {
        self.finished().is_none() &&
            self.slots.iter()
                .flat_map(|inner|inner)
                .any(|slot|slot.is_none())
    }
}

impl Index<(usize, usize)> for Field {
    type Output = Option<Mark>;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.slots[index.0][index.1]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Data)]
pub struct FieldMeta {
    field: Field,
    next_turn: Mark,
    written: Option<(usize, usize)>,
    active: bool,
}

impl FieldMeta {
    pub fn from_data(game_data: &GameData, position: (usize, usize)) -> Self {
        FieldMeta {
            field: *game_data.game[position],
            next_turn: game_data.next_turn,
            active: (game_data.next_field == Some(position) || game_data.next_field == None) &&
                game_data.game[position].playable() &&
                game_data.game.finished().is_none(),
            written: None,
        }
    }
    pub fn write_back(mut self, game_data: &mut GameData, field_position: (usize, usize)) {
        if let Some((x, y)) = self.written {
            self.field.set(x, y, self.next_turn);
            game_data.game.set(field_position.0, field_position.1, self.field);

            game_data.next_turn = if self.next_turn == Mark::Cross {
                Mark::Circle
            } else {
                Mark::Cross
            };
            game_data.next_field = if game_data.game[(x, y)].playable() {
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

#[derive(Clone, Eq, PartialEq, Debug, Data)]
pub struct Grid {
    slots: [[Arc<Field>; 3]; 3],
    finished: Option<Mark>,
}

impl Grid {
    pub fn new() -> Self {
        let row = [Arc::new(Field::new()), Arc::new(Field::new()), Arc::new(Field::new())];

        Grid {
            slots: [row.clone(), row.clone(), row],
            finished: None
        }
    }
    fn row_finished(&self, row: usize) -> Option<Mark> {
        if self[(0, row)].finished() == self[(1, row)].finished() && self[(1, row)].finished() == self[(2, row)].finished() {
            self[(0, row)].finished()
        } else {
            None
        }
    }
    fn column_finished(&self, column: usize) -> Option<Mark> {
        if self[(column, 0)].finished() == self[(column, 1)].finished() && self[(column, 1)].finished() == self[(column, 2)].finished() {
            self[(column, 0)].finished()
        } else {
            None
        }
    }
    fn diagonal_finished(&self) -> Option<Mark> {
        if (self[(0, 0)].finished() == self[(1, 1)].finished() && self[(1, 1)].finished() == self[(2, 2)].finished()) ||
            (self[(0, 2)].finished() == self[(1, 1)].finished() && self[(1, 1)].finished() == self[(2, 0)].finished()) {
            self[(1, 1)].finished()
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
    }
    pub fn finished(&self) -> Option<Mark> {
        self.finished
    }

    pub fn set(&mut self, x: usize, y: usize, field: Field) {
        self.slots[x][y] = Arc::new(field);
        self.calc_finished();
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Arc<Field>;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.slots[index.0][index.1]
    }
}

#[derive(Clone, Data, Lens)]
pub struct GameData {
    pub game: Grid,
    pub next_turn: Mark,
    pub next_field: Option<(usize, usize)>,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            game: Grid::new(),
            next_turn: Mark::Cross,
            next_field: None,
        }
    }
}