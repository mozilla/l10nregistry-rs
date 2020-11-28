use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::ParallelProblemSolver;

#[tokio::main]
async fn main() {
    let scenarios = get_scenarios();

    let scenario = scenarios
        .iter()
        .find(|s| s.name == "two-res-two-sources")
        .unwrap();

    let reg = scenario.get_l10nregistry();

    let mut gen =
        ParallelProblemSolver::new(scenario.res_ids.clone(), "en-US".parse().unwrap(), reg);

    while let Some(solution) = gen.next().await {
        println!("result: {:?}", solution);
    }
}
