use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use resizing_vec::ResizingVec;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::{prelude::*, BufReader};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ChainAccess");

    let root_rv = setup_rv();
    let root_hs = setup_hs();

    let upper = black_box(3);
    for locate in 0..upper {
        group.bench_with_input(
            BenchmarkId::new("ResizingVec", locate),
            &locate,
            |b, locate| {
                for channel in 0..upper {
                    b.iter(|| root_rv.get(channel, *locate))
                }
            },
        );
        group.bench_with_input(BenchmarkId::new("HashSet", locate), &locate, |b, locate| {
            for channel in 0..upper {
                b.iter(|| root_hs.get(channel, *locate))
            }
        });
    }
}

#[derive(Eq, Hash, PartialEq)]
struct LocateId(pub usize, pub usize);

pub struct Chain {
    channel: usize,
    locate: usize,
    hash_id: LocateId,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        self.locate == other.locate && self.channel == other.channel
    }
}
impl Eq for Chain {}
impl Hash for Chain {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.locate.hash(state);
        self.channel.hash(state);
    }
}

impl std::borrow::Borrow<LocateId> for Chain {
    fn borrow(&self) -> &LocateId {
        &self.hash_id
    }
}

fn base_setup() -> Vec<Chain> {
    let file = File::open("./nl.csv").unwrap();
    let reader = BufReader::new(file);

    let mut v = Vec::default();

    let mut header_seen = false;
    for line in reader.lines() {
        if !header_seen {
            header_seen = true;
            continue;
        }

        let line = line.unwrap();
        let splits = line.split(',').collect::<Vec<&str>>();

        v.push(Chain {
            channel: splits[1].parse().unwrap(),
            locate: splits[2].parse().unwrap(),
            hash_id: LocateId(splits[1].parse().unwrap(), splits[2].parse().unwrap()),
        });
    }

    v
}

struct RsRoot {
    dat: ResizingVec<ResizingVec<Chain>>,
}

impl RsRoot {
    pub fn get(&self, channel: usize, locate: usize) -> Option<&Chain> {
        if let Some(rv) = self.dat.get(channel) {
            return rv.get(locate);
        }

        None
    }
}

fn setup_rv() -> RsRoot {
    let data = base_setup();

    let mut root = RsRoot {
        dat: ResizingVec::new(),
    };

    for x in data {
        if root.dat.get(x.channel).is_none() {
            root.dat.insert(x.channel, ResizingVec::new());
        }

        root.dat.get_mut(x.channel).unwrap().insert(x.locate, x);
    }

    root
}

struct HsRoot {
    dat: HashSet<Chain>,
}

impl HsRoot {
    pub fn get(&self, channel: usize, locate: usize) -> Option<&Chain> {
        self.dat.get(&LocateId(channel, locate))
    }
}

fn setup_hs() -> HsRoot {
    let data = base_setup();

    let mut root = HsRoot {
        dat: HashSet::new(),
    };
    let len = data.len();
    for x in data {
        root.dat.insert(x);
    }

    if root.dat.len() != len {
        panic!("Data went missing")
    }

    root
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
