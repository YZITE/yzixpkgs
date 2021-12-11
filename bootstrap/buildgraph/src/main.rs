use std::{collections::HashMap, path::Path, stringify, sync::Arc};
use yzix_core::build_graph::{Edge, EdgeKind, Graph, Node, NodeKind};
use yzix_core::{pattern, store::{Dump, Hash}};

fn main() {
    let mut g = Graph::<()>::default();
    let mut logtag: u64 = 0;
    let mut names = HashMap::new();
    let mut next_logtag = move || {
        logtag += 1;
        logtag
    };

    // this macro avoida unnecessary repetition
    macro_rules! node_ {
        ($name:ident, $kind:expr) => {{
            names.insert(
                stringify!($name).to_string(),
                g.0.add_node(Node {
                    name: stringify!($name).to_string(),
                    kind: $kind,
                    logtag: next_logtag(),
                    rest: (),
                }),
            );
        }};
    }
    macro_rules! edge_ {
        (@ root) => {{ EdgeKind::Root }};
        (@ p; $ph:expr) => {{ EdgeKind::Placeholder($ph.to_string()) }};
        ($dst:expr, $src:expr, $($x:tt)*) => {{
            g.0.add_edge(names[stringify!($dst)], names[stringify!($src)], Edge {
                kind: edge_!(@ $($x)*),
                sel_output: Default::default(),
            })
        }};
    }

    node_!(
        reduce_sh,
        NodeKind::UnDump {
            dat: Arc::new(Dump::read_from_path(Path::new("reduce.sh")).unwrap()),
        }
    );
    node_!(
        alpine_root_pre,
        NodeKind::Require {
            hash: "m84OFxOfkVnnF7om15va9o1mgFcWD1TGH26ZhTLPuyg"
                .parse()
                .unwrap(),
        }
    );
    node_!(
        alpine_root,
        NodeKind::Run {
            command: pattern![I "/bin/busybox"; I "sh"; P "reduce"],
            envs: Default::default(),
            outputs: Default::default(),
        }
    );
    edge_!(alpine_root, reduce_sh, p;"reduce");
    edge_!(alpine_root, alpine_root_pre, root);
    node_!(gentoo_dl, NodeKind::Fetch {
        url: "https://mirror.ps.kz/gentoo/pub/releases/amd64/autobuilds/current-stage3-amd64-openrc/stage3-amd64-openrc-20211205T170532Z.tar.xz".try_into().unwrap(),
        hash: Some("EjqAlCYFC8RHyNRaIUTN2wFD3PO9Cz4h9vKQYg6mhDs".parse().unwrap()),
    });
    node_!(
        unpack_txz,
        NodeKind::UnDump {
            dat: Arc::new(Dump::read_from_path(Path::new("unpack-txz.sh")).unwrap()),
        }
    );
    node_!(
        gentoo_root_pre,
        NodeKind::Run {
            command: pattern![I "/bin/busybox"; I "sh"; P "utxzsh"; P "archive"],
            envs: Default::default(),
            outputs: Default::default(),
        }
    );
    edge_!(gentoo_root_pre, alpine_root, root);
    edge_!(gentoo_root_pre, unpack_txz, p;"utxzsh");
    edge_!(gentoo_root_pre, gentoo_dl, p;"archive");
    node_!(
        gentoo_root,
        NodeKind::Run {
            command: pattern![I "/bin/sh"; P "reduce"],
            envs: Default::default(),
            outputs: Default::default(),
        }
    );
    edge_!(gentoo_root, reduce_sh, p;"reduce");
    edge_!(gentoo_root, gentoo_root_pre, root);

    // LFS
    for i in std::fs::read_to_string("./lfs-wget-list").expect("unable to use lfs-wget-list").lines() {
        let url: yzix_core::Url = i.parse().unwrap();
        let name = url.path_segments().unwrap().last().unwrap();
        let logtag = next_logtag();
        eprintln!("{}\t{}\t=> {}", logtag, name, url);
        names.insert(
            name.to_string(),
            g.0.add_node(Node {
                name: name.to_string(),
                kind: NodeKind::Fetch {
                    url,
                    hash: None,
                },
                logtag,
                rest: (),
            })
        );
    }

    let hashes = std::fs::read_to_string("./hashes").expect("unable to use hashes").lines().flat_map(|i| i.find(':').map(|x| (&i[..x], &i[x+1..]))).map(|(i, j)| (i.parse().unwrap(), j.parse().unwrap())).collect::<HashMap<u64, Hash>>();
    for (logtag, outhash) in hashes {
        let j = match g.0.node_indices().find(|&j| g.0[j].logtag == logtag) {
            Some(x) => x,
            // this shouldn't happen
            None => continue,
        };
        if let NodeKind::Fetch { ref mut hash, .. } = &mut g.0[j].kind {
            *hash = Some(outhash);
        }
    }

    println!(
        "{}",
        serde_json::to_string(&g).expect("serialization failed")
    );
}
