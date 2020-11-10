use super::ProblemSolver;
use crate::fluent::FluentBundle;

// Streaming Iterator (return refs to its own field)
// Solver with backtracking

pub trait SerialProblemSolver {
    fn next(&mut self) -> Option<&Vec<usize>>;
    fn next_bundle(&mut self) -> Option<FluentBundle>;
}

impl SerialProblemSolver for ProblemSolver {
    #[inline]
    fn next(&mut self) -> Option<&Vec<usize>> {
        if self.solution.is_complete() {
            if !self.solution.bail() {
                return None;
            }
        }
        loop {
            if !self.test_solution() {
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            if self.solution.is_complete() {
                return Some(&self.solution.candidate);
            }
            if !self.solution.advance() {
                return None;
            }
        }
    }

    #[inline]
    fn next_bundle(&mut self) -> Option<FluentBundle> {
        if self.solution.is_complete() {
            if !self.solution.bail() {
                return None;
            }
        }
        loop {
            if !self.test_solution() {
                if !self.solution.bail() {
                    return None;
                }
                continue;
            }
            if self.solution.is_complete() {
                return Some(self.get_bundle());
            }
            if !self.solution.advance() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::testing::get_scenarios;
    use unic_langid::LanguageIdentifier;

    #[test]
    fn test_sync() {
        let scenarios = get_scenarios();

        let langid: LanguageIdentifier = "en-US".parse().unwrap();

        for scenario in scenarios {
            let reg = scenario.get_l10nregistry();
            let mut gen = ProblemSolver::new(scenario.res_ids.clone(), langid.clone(), reg);

            if let Some(solutions) = &scenario.solutions {
                let mut i = 0;
                while let Some(solution) = gen.next() {
                    assert!(
                        solutions.len() > i,
                        "too many solutions, scenario: {}",
                        scenario.name
                    );
                    assert_eq!(solution, solutions.get(i).unwrap());
                    i += 1;
                }
                assert_eq!(
                    i,
                    solutions.len(),
                    "too few solutions, scenario: {}",
                    scenario.name
                );
            }
        }
    }
}
