use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use l10nregistry::iter::synchronous::{SyncSourcesGenerator, Tester};

pub struct MockFileSource {
    missing: Vec<(usize, usize)>,
}

impl MockFileSource {
    pub fn new(missing: Vec<(usize, usize)>) -> Self {
        Self { missing }
    }
}

impl Tester for MockFileSource {
    fn test_cell(&self, _resource: usize, _source: usize) -> Option<bool> {
        None
    }

    fn get_cell(&mut self, resource: usize, source: usize) -> bool {
        for cell in &self.missing {
            if cell == &(resource, source) {
                return false;
            }
        }
        return true;
    }
}

fn sync_bench(c: &mut Criterion) {
    c.bench_function("sync/small", move |b| {
        b.iter(|| {
            let mut fs = MockFileSource::new(vec![]);

            let resources = vec![0, 1, 2];
            let sources = vec![0, 1];

            let mut gen = SyncSourcesGenerator::new(resources, sources, &mut fs);
            while let Some(_) = gen.next() {}
        })
    });

    c.bench_function("sync/preferences/two-sources", move |b| {
        b.iter(|| {
            let mut fs = MockFileSource::new(vec![]);

            let resources = vec![0, 1, 2, 3, 4, 5];
            let sources = vec![0, 1];

            let mut gen = SyncSourcesGenerator::new(resources, sources, &mut fs);
            while let Some(_) = gen.next() {}
        })
    });

    c.bench_function("sync/preferences/langpack", move |b| {
        b.iter(|| {
            let mut fs = MockFileSource::new(vec![]);

            let resources = vec![0, 1, 2, 3, 4, 5];
            let sources = vec![0, 1, 2, 3];

            let mut gen = SyncSourcesGenerator::new(resources, sources, &mut fs);
            while let Some(_) = gen.next() {}
        })
    });
}

criterion_group!(benches, sync_bench);
criterion_main!(benches);
