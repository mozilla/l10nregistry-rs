use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::SerialProblemSolver;

fn main() {
    let scenarios = get_scenarios();

    let scenario = scenarios.iter().find(|s| s.name == "preferences").unwrap();

    let reg = scenario.get_l10nregistry();

    let mut gen = SerialProblemSolver::new(scenario.res_ids.clone(), "en-US".parse().unwrap(), reg);

    for res_id in &scenario.res_ids {
        gen.add_key(res_id.to_string());
    }

    // gen.prefetch();
    // let now = std::time::Instant::now();
    // let result = gen.next();
    // println!("Elapsed: {} ns", now.elapsed().as_nanos());

    while let Some(solution) = gen.next() {
        println!("result: {:?}", solution);
    }
}
