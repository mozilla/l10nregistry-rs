use futures::stream::Collect;
use futures::stream::FuturesOrdered;
use futures::stream::StreamExt;
use l10nregistry::solver::testing::get_scenarios;
use l10nregistry::solver::{AsyncTester, ParallelProblemSolver, SerialProblemSolver, SyncTester};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct MockTester {
    values: Vec<Vec<bool>>,
}

impl SyncTester for MockTester {
    fn test_sync(&self, res_idx: usize, source_idx: usize) -> bool {
        self.values[res_idx][source_idx]
    }
}

pub struct SingleTestResult(bool);

impl Future for SingleTestResult {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.into()
    }
}

pub type ResourceSetStream = Collect<FuturesOrdered<SingleTestResult>, Vec<bool>>;

pub struct TestResult(ResourceSetStream);

impl std::marker::Unpin for TestResult {}

impl Future for TestResult {
    type Output = Vec<bool>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pinned = Pin::new(&mut self.0);
        pinned.poll(cx)
    }
}

impl AsyncTester for MockTester {
    type Result = TestResult;

    fn test_async(&self, query: Vec<(usize, usize)>) -> Self::Result {
        let futures = query
            .into_iter()
            .map(|(res_idx, source_idx)| SingleTestResult(self.test_sync(res_idx, source_idx)))
            .collect::<Vec<_>>();
        TestResult(futures.into_iter().collect::<FuturesOrdered<_>>().collect())
    }
}

struct TestStream<'t> {
    solver: ParallelProblemSolver<MockTester>,
    tester: &'t MockTester,
}

impl<'t> TestStream<'t> {
    pub fn new(solver: ParallelProblemSolver<MockTester>, tester: &'t MockTester) -> Self {
        Self { solver, tester }
    }
}

impl<'t> futures::stream::Stream for TestStream<'t> {
    type Item = Vec<usize>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let tester = self.tester;
        let solver = &mut self.solver;
        let pinned = std::pin::Pin::new(solver);
        pinned.poll_next(cx, tester)
    }
}

#[test]
fn serial_scenarios_test() {
    for scenario in get_scenarios() {
        let tester = MockTester {
            values: scenario.values.clone(),
        };

        let mut gen = SerialProblemSolver::new(scenario.width, scenario.depth);

        let mut idx = 0;
        while let Some(candidate) = gen.next(&tester) {
            let expected = scenario
                .solutions
                .get(idx)
                .expect("SerialProblemSolver produced superfluous solution");
            assert_eq!(
                candidate,
                expected.as_slice(),
                "SerialProblemSolver produced wrong solution"
            );
            idx += 1;
        }

        assert_eq!(
            scenario.solutions.len(),
            idx,
            "SerialProblemSolver produced too few solutions"
        );
    }
}

#[tokio::test]
async fn parallel_scenarios_test() {
    for scenario in get_scenarios() {
        let tester = MockTester {
            values: scenario.values.clone(),
        };
        let gen = ParallelProblemSolver::new(scenario.width, scenario.depth);

        let mut idx = 0;
        let mut t = TestStream::new(gen, &tester);
        while let Some(candidate) = t.next().await {
            let expected = scenario
                .solutions
                .get(idx)
                .expect("ParallelSolver produced superfluous solution");
            assert_eq!(
                candidate,
                expected.as_slice(),
                "ParallelSolver produced wrong solution"
            );
            idx += 1;
        }

        assert_eq!(
            scenario.solutions.len(),
            idx,
            "ParallelSolver produced too few solutions"
        );
    }
}
