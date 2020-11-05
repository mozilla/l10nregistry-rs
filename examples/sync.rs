use tracing::{info, Level};

use l10nregistry::iter::synchronous::{SyncSourcesGenerator, Tester};

enum FilesList {
    Missing(Vec<(usize, usize)>),
    Present(Vec<(usize, usize)>),
}
struct MockFileSource {
    list: FilesList,
}

impl MockFileSource {
    pub fn new(list: FilesList) -> Self {
        Self { list }
    }
}

impl Tester for MockFileSource {
    fn test_cell(&self, _resource: usize, _source: usize) -> Option<bool> {
        None
    }

    fn get_cell(&mut self, resource: usize, source: usize) -> bool {
        match &self.list {
            FilesList::Missing(missing) => {
                for cell in missing {
                    if cell == &(resource, source) {
                        return false;
                    }
                }
                return true;
            }
            FilesList::Present(list) => {
                for cell in list {
                    if cell == &(resource, source) {
                        return true;
                    }
                }
                return false;
            }
        }
    }
}

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        // builds the subscriber.
        .finish();

    let mut fs = MockFileSource::new(FilesList::Missing(vec![]));

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // let resources = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    // let sources = vec![0, 1, 2, 3];
    let resources = vec![0, 1, 2];
    let sources = vec![0, 1];
    let mut gen = SyncSourcesGenerator::new(resources, sources, &mut fs);
    while let Some(order) = gen.next() {
        info!("result: {:?}", order);
    }
}
