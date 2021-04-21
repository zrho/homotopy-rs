use crate::common::{Height, SingularHeight};
use crate::diagram::{Diagram, DiagramN};
use crate::rewrite::{Cospan, Rewrite, RewriteN};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{Into, TryFrom, TryInto};
use std::rc::Rc;

/// A degeneracy map which keeps track of a subset of identity levels in a diagram.
///
/// Degeneracy maps are non-globular in order to support the normalization of a diagram's
/// boundaries. Since we only normalize globular diagrams however, the regular slices of any
/// degeneracy map that we construct are determined by the regular slices of the target diagram.
/// Therefore this data type only stores the singular slices of any degeneracy map.
#[derive(Debug, Clone)]
enum Degeneracy {
    Identity,
    Degeneracy(Vec<SingularHeight>, Vec<Rc<Degeneracy>>),
}

impl Degeneracy {
    fn new(trivial: Vec<SingularHeight>, slices: Vec<Rc<Self>>) -> Self {
        if trivial.is_empty() && slices.iter().all(|slice| slice.is_identity()) {
            Self::Identity
        } else {
            Self::Degeneracy(trivial, slices)
        }
    }

    fn singular_preimage(&self, i: SingularHeight) -> Height {
        match self {
            Self::Identity => Height::Singular(i),
            Self::Degeneracy(trivial, _) => {
                for (count, trivial) in trivial.iter().enumerate() {
                    match trivial.cmp(&i) {
                        Ordering::Less => {}
                        Ordering::Equal => return Height::Regular(i - count),
                        Ordering::Greater => return Height::Singular(i - count),
                    }
                }

                Height::Singular(i - trivial.len())
            }
        }
    }

    fn is_identity(&self) -> bool {
        match self {
            Self::Identity => true,
            Self::Degeneracy(_, _) => false,
        }
    }

    /// Converts this degeneracy map into a rewrite, under the assumption that it represents a
    /// globular degeneracy map. This function is supposed to be called by the `normalize_singular`
    /// function which does not normalize regular levels. The assumption of globularity is not
    /// checked.
    fn to_rewrite(&self, source: &Diagram, target: &Diagram) -> Rewrite {
        assert_eq!(source.dimension(), target.dimension());

        let (trivial, slices) = match self {
            Self::Identity => return Rewrite::identity(source.dimension()),
            Self::Degeneracy(trivial, slices) => (trivial, slices),
        };

        let source: &DiagramN = source.try_into().unwrap();
        let target: &DiagramN = target.try_into().unwrap();

        let rewrite_simple = RewriteN::make_degeneracy(source.dimension(), &trivial);
        let middle = source.clone().rewrite_forward(&rewrite_simple);
        let middle_slices: Vec<_> = middle.slices().collect();
        let target_slices: Vec<_> = target.slices().collect();

        let rewrite_slices: Vec<_> = slices
            .iter()
            .enumerate()
            .map(|(i, slice)| {
                let middle_slice = &middle_slices[Height::Singular(i).to_int()];
                let target_slice = &target_slices[Height::Singular(i).to_int()];
                vec![slice.to_rewrite(middle_slice, target_slice)]
            })
            .collect();
        let rewrite_parallel = RewriteN::from_slices(
            middle.dimension(),
            middle.cospans(),
            target.cospans(),
            rewrite_slices,
        );

        RewriteN::compose(&rewrite_simple, &rewrite_parallel)
            .unwrap()
            .into()
    }

    fn slice_into(&self, target_height: SingularHeight) -> &Self {
        match self {
            Self::Identity => &Self::Identity,
            Self::Degeneracy(_, slices) => &slices[target_height],
        }
    }
}

#[derive(Debug, Clone)]
struct SinkArrow {
    source: Diagram,
    degeneracy: Rc<Degeneracy>,
    middle: Diagram,
    rewrite: Rewrite,
}

impl SinkArrow {
    fn singular_preimage(&self, target_height: SingularHeight) -> Vec<SingularHeight> {
        let mut preimage = Vec::new();
        let rewrite: &RewriteN = (&self.rewrite).try_into().unwrap();

        for middle_height in rewrite.singular_preimage(target_height) {
            match self.degeneracy.singular_preimage(middle_height) {
                Height::Singular(source_height) => preimage.push(source_height),
                Height::Regular(_) => {}
            }
        }

        preimage
    }

    fn slices_into(&self, target_height: SingularHeight) -> Vec<(SingularHeight, Self)> {
        let rewrite: &RewriteN = (&self.rewrite).try_into().unwrap();
        let source = <&DiagramN>::try_from(&self.source).ok().unwrap();
        let middle = <&DiagramN>::try_from(&self.middle).ok().unwrap();
        let mut slices = Vec::new();

        for middle_height in rewrite.singular_preimage(target_height) {
            let source_height = match self.degeneracy.singular_preimage(middle_height) {
                Height::Singular(source_height) => source_height,
                Height::Regular(_) => continue,
            };

            let slice_rewrite = rewrite.slice(middle_height);
            let slice_middle = middle.slice(Height::Singular(middle_height)).unwrap();
            let slice_degeneracy = self.degeneracy.slice_into(middle_height).clone();
            let slice_source = source.slice(Height::Singular(source_height)).unwrap();
            slices.push((
                source_height,
                Self {
                    source: slice_source,
                    rewrite: slice_rewrite,
                    middle: slice_middle,
                    degeneracy: Rc::new(slice_degeneracy),
                },
            ));
        }

        slices
    }
}

pub fn normalize(diagram: &Diagram) -> Diagram {
    // TODO: Cache by hash

    fn normalize_regular(regular: &Diagram) -> (Diagram, Rc<Degeneracy>) {
        let output = normalize_relative(regular, &[], normalize_regular);
        (output.diagram, output.degeneracy)
    }

    normalize_relative(diagram, &[], normalize_regular).diagram
}

pub fn normalize_singular(diagram: &Diagram) -> Rewrite {
    fn normalize_regular(regular: &Diagram) -> (Diagram, Rc<Degeneracy>) {
        (regular.clone(), Rc::new(Degeneracy::Identity))
    }

    let output = normalize_relative(diagram, &[], normalize_regular);
    output.degeneracy.to_rewrite(&output.diagram, diagram)
}

struct Output {
    factors: Vec<Rewrite>,
    degeneracy: Rc<Degeneracy>,
    diagram: Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Factor {
    Forward(SingularHeight),
    Backward(SingularHeight),
    Slice(usize, SingularHeight),
}

fn normalize_relative<R>(diagram: &Diagram, sink: &[SinkArrow], mut normalize_regular: R) -> Output
where
    R: FnMut(&Diagram) -> (Diagram, Rc<Degeneracy>) + Copy,
{
    use Height::{Regular, Singular};

    // Base case for 0-dimensional diagrams.
    let diagram = match diagram {
        Diagram::Diagram0(_) => {
            let factors = sink.iter().map(|input| input.rewrite.clone()).collect();
            return Output {
                factors,
                degeneracy: Rc::new(Degeneracy::Identity),
                diagram: diagram.clone(),
            };
        }
        Diagram::DiagramN(d) => d,
    };

    let slices: Vec<_> = diagram.slices().collect();

    let mut degeneracies: HashMap<Height, Rc<Degeneracy>> = HashMap::new();
    let mut factors: HashMap<Factor, Rewrite> = HashMap::new();
    let mut regular: Vec<Diagram> = Vec::new();

    // Normalize the regular levels
    for height in 0..=diagram.size() {
        let slice = &slices[Regular(height).to_int()];
        let (normalized, degeneracy) = normalize_regular(slice);
        regular.push(normalized);
        degeneracies.insert(Regular(height), degeneracy);
    }

    // Construct the subproblems
    let mut subproblems: Vec<Vec<SinkArrow>> = (0..diagram.size()).map(|_| Vec::new()).collect();
    let mut roles: Vec<Vec<Factor>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    for (i, cospan) in diagram.cospans().iter().enumerate() {
        subproblems[i].push(SinkArrow {
            source: regular[i].clone(),
            degeneracy: degeneracies[&Regular(i)].clone(),
            middle: slices[Regular(i).to_int()].clone(),
            rewrite: cospan.forward.clone(),
        });

        roles[i].push(Factor::Forward(i));

        subproblems[i].push(SinkArrow {
            source: regular[i + 1].clone(),
            degeneracy: degeneracies[&Regular(i + 1)].clone(),
            middle: slices[Regular(i + 1).to_int()].clone(),
            rewrite: cospan.backward.clone(),
        });

        roles[i].push(Factor::Backward(i));
    }

    for (i, input) in sink.iter().enumerate() {
        for target in 0..diagram.size() {
            for (j, slice) in input.slices_into(target) {
                subproblems[target].push(slice);
                roles[target].push(Factor::Slice(i, j));
            }
        }
    }

    // Solve the subproblems
    for target_height in 0..diagram.size() {
        let slice = &slices[Height::Singular(target_height).to_int()];
        let output = normalize_relative(slice, &subproblems[target_height], normalize_regular);
        degeneracies.insert(Singular(target_height), output.degeneracy);
        factors.extend(
            roles[target_height]
                .iter()
                .zip(output.factors)
                .map(|(role, factor)| (*role, factor)),
        );
    }

    // Find the trivial heights
    let trivial: Vec<SingularHeight> = (0..diagram.size())
        .filter(|target_height| {
            roles[*target_height].iter().all(|f| match f {
                Factor::Forward(_) | Factor::Backward(_) => factors[f].is_identity(),
                Factor::Slice(_, _) => false,
            })
        })
        .collect();

    // Assemble the normalized parts, filtering out the trivial levels.
    let normalized_cospans: Vec<_> = (0..diagram.size())
        .filter(|height| !trivial.iter().any(|t| t == height))
        .map(|height| Cospan {
            forward: factors[&Factor::Forward(height)].clone(),
            backward: factors[&Factor::Backward(height)].clone(),
        })
        .collect();

    let normalized_factors = sink
        .iter()
        .enumerate()
        .map(|(i, input)| {
            let source: &DiagramN = (&input.source).try_into().unwrap();

            RewriteN::from_slices(
                diagram.dimension(),
                source.cospans(),
                &normalized_cospans,
                (0..diagram.size())
                    .filter(|height| !trivial.iter().any(|t| t == height))
                    .map(|target_height| {
                        input
                            .singular_preimage(target_height)
                            .into_iter()
                            .map(|source_height| factors[&Factor::Slice(i, source_height)].clone())
                            .collect()
                    })
                    .collect(),
            )
            .into()
        })
        .collect();

    let normalized_diagram = DiagramN::new_unsafe(regular[0].clone(), normalized_cospans).into();

    let degeneracy = Degeneracy::new(
        trivial,
        (0..diagram.size())
            .map(|i| degeneracies[&Singular(i)].clone())
            .collect(),
    );

    Output {
        factors: normalized_factors,
        diagram: normalized_diagram,
        degeneracy: Rc::new(degeneracy),
    }
}