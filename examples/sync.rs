use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::SerialProblemSolver;

fn main() {
    let scenarios = get_scenarios();

    let scenario = scenarios
        .iter()
        .find(|s| s.name == "one-res-two-sources")
        .unwrap();

    let reg = scenario.get_l10nregistry();

    let mut gen = SerialProblemSolver::new(scenario.res_ids.clone(), "en-US".parse().unwrap(), reg);

    while let Some(solution) = gen.next() {
        println!("result: {:?}", solution);
    }
}
