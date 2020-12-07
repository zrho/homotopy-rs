use crate::geometry::Point;
use homotopy_core::common::*;
use homotopy_core::diagram::*;
use serde::Serialize;
use std::convert::*;
use thiserror::Error;

#[derive(Debug)]
struct Constraint(Vec<(usize, usize)>, (usize, usize));

impl Constraint {
    fn build(diagram: &DiagramN) -> Vec<Self> {
        let mut constraints = Vec::new();
        let cospans = diagram.cospans();
        let slices: Vec<_> = diagram.slices().collect();

        for i in 0..diagram.size() {
            let cospan = &cospans[i];
            let forward = cospan.forward.to_n().unwrap();
            let backward = cospan.backward.to_n().unwrap();
            let regular0: &DiagramN = (&slices[i * 2]).try_into().unwrap();
            let singular: &DiagramN = (&slices[i * 2 + 1]).try_into().unwrap();
            let regular1: &DiagramN = (&slices[i * 2 + 2]).try_into().unwrap();

            use Height::*;

            constraints.extend((0..singular.size()).map(|x| {
                Constraint(
                    forward
                        .singular_preimage(x)
                        .map(|xp| (Singular(xp).to_int(), Regular(i).to_int()))
                        .collect(),
                    (Singular(x).to_int(), Singular(i).to_int()),
                )
            }));

            constraints.extend((0..singular.size()).map(|x| {
                Constraint(
                    backward
                        .singular_preimage(x)
                        .map(|xp| (Singular(xp).to_int(), Regular(i + 1).to_int()))
                        .collect(),
                    (Singular(x).to_int(), Singular(i).to_int()),
                )
            }));

            constraints.extend((0..regular0.size() + 1).map(|x| {
                Constraint(
                    forward
                        .regular_preimage(x)
                        .map(|xp| (Regular(xp).to_int(), Singular(i).to_int()))
                        .collect(),
                    (Regular(x).to_int(), Regular(i).to_int()),
                )
            }));

            constraints.extend((0..regular1.size() + 1).map(|x| {
                Constraint(
                    backward
                        .regular_preimage(x)
                        .map(|xp| (Regular(xp).to_int(), Singular(i).to_int()))
                        .collect(),
                    (Regular(x).to_int(), Regular(i + 1).to_int()),
                )
            }));
        }

        constraints
    }
}

/// Position store used in the [Solver].
struct Positions(Vec<Vec<f32>>);

impl Positions {
    fn get(&self, point: (usize, usize)) -> f32 {
        self.0[point.1][point.0]
    }

    fn set(&mut self, point: (usize, usize), value: f32) {
        self.0[point.1][point.0] = value;
    }

    fn propagate(&mut self, constraint: &Constraint) -> bool {
        let source_sum = constraint.0.iter().map(|p| self.get(*p)).sum::<f32>();

        let target_pos = self.get(constraint.1);

        let diff = source_sum / (constraint.0.len() as f32) - target_pos;

        if diff > 0.01 {
            self.set(constraint.1, self.get(constraint.1) + diff);
            true
        } else if diff < -0.01 {
            constraint
                .0
                .iter()
                .for_each(|p| self.set(*p, self.get(*p) - diff));
            true
        } else {
            false
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("layout solver expected a diagram of dimension two or higher.")]
    Dimension,
}

/// Iterative solver for 2-dimensional diagram layouts.
pub struct Solver {
    positions: Positions,
    constraints: Vec<Constraint>,
}

impl Solver {
    pub fn new(diagram: DiagramN) -> Result<Self, Error> {
        if diagram.dimension() < 2 {
            return Err(Error::Dimension);
        }

        let positions = Positions(
            diagram
                .slices()
                .map(|slice| {
                    (0..DiagramN::try_from(slice).unwrap().size() * 2 + 1)
                        .map(|i| i as f32)
                        .collect()
                })
                .collect(),
        );

        let constraints = Constraint::build(&diagram);

        Ok(Solver {
            positions,
            constraints,
        })
    }

    fn propagate(&mut self) -> bool {
        let mut changed = false;

        for constraint in &self.constraints {
            changed = self.positions.propagate(constraint);
        }

        changed
    }

    fn distance(&mut self) -> bool {
        let mut changed = false;

        for y in 0..self.positions.0.len() {
            for x in 1..self.positions.0[y].len() {
                let first = self.positions.0[y][x - 1];
                let second = self.positions.0[y][x];

                if second < first + 1.0 {
                    self.positions.0[y][x] = first + 1.0;
                    changed = true;
                }
            }
        }

        changed
    }

    pub fn step(&mut self) -> bool {
        let mut changed = false;
        changed = self.distance() || changed;
        changed = self.propagate() || changed;
        changed
    }

    /// Run as many iterations of the solver until the positions stabilize or a maximum number of
    /// steps is reached. Then return the number of steps.
    pub fn solve(&mut self, max_steps: usize) -> usize {
        for step in 0..max_steps {
            if !self.step() {
                return step;
            }
        }

        max_steps
    }

    pub fn finish(self) -> Layout {
        Layout(self.positions.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Layout(Vec<Vec<f32>>);

impl Layout {
    /// Solve for a diagram's layout with a given maximum number of iterations.
    ///
    /// Expects diagrams of dimension two or higher.
    pub fn new(diagram: DiagramN, max_steps: usize) -> Result<Layout, Error> {
        let mut solver = Solver::new(diagram)?;
        solver.solve(max_steps);
        Ok(solver.finish())
    }

    /// The position of a logical point in the layout.
    pub fn get(&self, x: SliceIndex, y: SliceIndex) -> Option<Point> {
        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => self.0.first()?,
            SliceIndex::Boundary(Boundary::Target) => self.0.last()?,
            SliceIndex::Interior(height) => self.0.get(height.to_int())?,
        };

        let y_pos = match y {
            SliceIndex::Boundary(Boundary::Source) => 0.0,
            SliceIndex::Boundary(Boundary::Target) => self.0.len() as f32 + 1.0,
            SliceIndex::Interior(height) => height.to_int() as f32 + 1.0,
        };

        let x_pos = match x {
            SliceIndex::Boundary(Boundary::Source) => 0.0,
            SliceIndex::Boundary(Boundary::Target) => slice.last()? + 2.0,
            SliceIndex::Interior(height) => slice.get(height.to_int())? + 1.0,
        };

        Some((x_pos, y_pos).into())
    }
}

mod tests {
    use super::*;

    fn example_assoc() -> DiagramN {
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let m = Generator {
            id: 2,
            dimension: 2,
        };

        let fd = DiagramN::new(f, x, x).unwrap();
        let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
        let md = DiagramN::new(m, ffd, fd).unwrap();
        md.attach(md.clone(), Boundary::Source, &[1]).unwrap()
    }

    #[test]
    fn test_assoc() {
        let diagram = example_assoc();
        let mut solver = Solver::new(diagram).unwrap();
        solver.solve(10);
        let _layout = solver.finish();

        // TODO: Write test
    }
}