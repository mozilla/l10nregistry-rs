use tracing::{info, Level};

use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::ParallelProblemSolver;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        // builds the subscriber.
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let scenarios = get_scenarios();

    let scenario = scenarios.iter().find(|s| s.name == "incomplete").unwrap();

    let reg = scenario.get_l10nregistry();

    let mut gen =
        ParallelProblemSolver::new(scenario.res_ids.clone(), "en-US".parse().unwrap(), reg);

    while let Some(solution) = gen.next().await {
        info!("result: {:?}", solution);
    }
}
