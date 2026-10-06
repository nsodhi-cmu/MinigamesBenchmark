use super::{BOARDS, SIDE, Twins, dots, middle};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

const CELLS: usize = SIDE * SIDE;

fn beside(cell: usize) -> impl Iterator<Item = usize> {
    let (row, column) = (cell / SIDE, cell % SIDE);
    [
        (row > 0).then(|| cell - SIDE),
        (row + 1 < SIDE).then(|| cell + SIDE),
        (column > 0).then(|| cell - 1),
        (column + 1 < SIDE).then(|| cell + 1),
    ]
    .into_iter()
    .flatten()
}

/// Every way of joining the pairs without crossing, whether or not it covers
/// the board, stopping at two.
struct Search {
    dots: Vec<[usize; 2]>,
    taken: [bool; CELLS],
    lines: Vec<Vec<usize>>,
    found: Vec<Vec<Vec<usize>>>,
}

impl Search {
    fn open(&self, color: usize) -> bool {
        self.lines[color].last() != Some(&self.dots[color][1])
    }

    /// Whether every unfinished line can still reach its other dot.
    fn hopeful(&self) -> bool {
        (0..self.dots.len()).filter(|color| self.open(*color)).all(|color| {
            let (head, goal) = (*self.lines[color].last().unwrap(), self.dots[color][1]);
            let mut seen = [false; CELLS];
            let mut stack = vec![head];
            while let Some(cell) = stack.pop() {
                for next in beside(cell) {
                    if next == goal {
                        return true;
                    }
                    if !self.taken[next] && !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
            false
        })
    }

    fn run(&mut self) {
        if self.found.len() == 2 {
            return;
        }
        let open: Vec<usize> = (0..self.dots.len()).filter(|color| self.open(*color)).collect();
        if open.is_empty() {
            self.found.push(self.lines.clone());
            return;
        }
        let moves = |color: usize| {
            let head = *self.lines[color].last().unwrap();
            beside(head)
                .filter(|next| !self.taken[*next] || *next == self.dots[color][1])
                .collect::<Vec<_>>()
        };
        let color = *open.iter().min_by_key(|color| moves(**color).len()).unwrap();
        for next in moves(color) {
            let goal = next == self.dots[color][1];
            self.taken[next] = true;
            self.lines[color].push(next);
            if self.hopeful() {
                self.run();
            }
            self.lines[color].pop();
            if !goal {
                self.taken[next] = false;
            }
        }
    }
}

fn answers(board: usize) -> Vec<Vec<Vec<usize>>> {
    let dots = dots(board);
    let mut taken = [false; CELLS];
    for cell in dots.iter().flatten() {
        taken[*cell] = true;
    }
    let lines = dots.iter().map(|pair| vec![pair[0]]).collect();
    let mut search = Search { dots, taken, lines, found: Vec::new() };
    search.run();
    search.found
}

fn act(game: &mut Twins, input: &mut Input, action: Action) -> Outcome {
    input.apply(action);
    let outcome = game.update(input);
    input.settle();
    outcome
}

/// Each board has one way to join its pairs, even before asking that the lines
/// cover it, so a board cannot be joined with squares left over.
#[test]
fn every_board_has_exactly_one_way_to_join_its_pairs_and_it_covers_the_board() {
    for (board, rows) in BOARDS.iter().enumerate() {
        assert_eq!(dots(board).len() * 2, rows.concat().matches(|c| c != '.').count());
        let found = answers(board);
        assert_eq!(found.len(), 1, "board {} can be joined {} ways", board + 1, found.len());
        let covered: usize = found[0].iter().map(Vec::len).sum();
        assert_eq!(covered, CELLS, "board {} is joined with squares left over", board + 1);
    }
}

/// Drawing each answer with the pointer, jumping straight to each corner of a
/// line rather than square by square, clears all three boards.
#[test]
fn drawing_each_answer_with_the_pointer_wins() {
    let mut game = Twins::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for board in 0..BOARDS.len() {
        assert_eq!(game.board, board);
        for line in answers(board).remove(0) {
            let (x, y) = middle(line[0]);
            act(&mut game, &mut input, Action::Move { x, y });
            let here = input.cursor();
            act(
                &mut game,
                &mut input,
                Action::Press { button: Button::Left, x: here.0, y: here.1 },
            );
            for (i, cell) in line.iter().enumerate().skip(1) {
                let turns = line.get(i + 1).is_none_or(|after| {
                    after % SIDE != line[i - 1] % SIDE && after / SIDE != line[i - 1] / SIDE
                });
                if turns {
                    let (x, y) = middle(*cell);
                    act(&mut game, &mut input, Action::Move { x, y });
                }
            }
            let here = input.cursor();
            outcome = act(
                &mut game,
                &mut input,
                Action::Release { button: Button::Left, x: here.0, y: here.1 },
            );
        }
    }
    assert_eq!(outcome, Outcome::Won);
}
