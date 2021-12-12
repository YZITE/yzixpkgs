use std::{collections::HashMap, path::Path, stringify, sync::Arc};
use yzix_core::build_graph::{Edge, EdgeKind, Graph, Node, NodeKind};
use yzix_core::{pattern, store::{Dump, Hash}};

fn dlocal<P: std::convert::AsRef<Path>>(x: P) -> NodeKind {
    NodeKind::UnDump {
        dat: Arc::new(Dump::read_from_path(x.as_ref()).unwrap()),
    }
}

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
        (@@ root) => {{ EdgeKind::Root }};
        (@@ $ph:literal) => {{ EdgeKind::Placeholder($ph.to_string()) }};
        (@ node $src:ident) => {{ stringify!($src) }};
        (@ node $src:literal) => {{ $src }};
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
        ($name:ident, $kind:expr, $(( $src:tt, $($x:tt)* )),* $(,)?) => {{
            let curdst = g.0.add_node(Node {
                name: stringify!($name).to_string(),
                kind: $kind,
                logtag: next_logtag(),
                rest: (),
            });
            names.insert(
                stringify!($name).to_string(),
                curdst,
            );
            $(
                g.0.add_edge(curdst, names[node_!(@ node $src)], Edge {
                    kind: node_!(@@ $($x)*),
                    sel_output: Default::default(),
                });
            )*
        }};
    }

    node_!(reduce_sh, dlocal("reduce.sh"));
    node_!(
        alpine_root_pre,
        NodeKind::Require {
            hash: "JzuKGzdi6Nliq4xWr0fX6CT+DPG5pX65c91PdhR8GAU"
                .parse()
                .unwrap(),
        },
    );
    node_!(
        alpine_root,
        NodeKind::Run {
            command: pattern![I "/bin/busybox"; I "sh"; P "reduce"],
            envs: Default::default(),
            outputs: Default::default(),
        },
        (reduce_sh, "reduce"),
        (alpine_root_pre, root),
    );
    node_!(gentoo_dl, NodeKind::Fetch {
        url: "https://mirror.ps.kz/gentoo/pub/releases/amd64/autobuilds/current-stage3-amd64-openrc/stage3-amd64-openrc-20211205T170532Z.tar.xz".try_into().unwrap(),
        hash: Some("EjqAlCYFC8RHyNRaIUTN2wFD3PO9Cz4h9vKQYg6mhDs".parse().unwrap()),
    });
    node_!(unpack_txz, dlocal("unpack-txz.sh"));
    node_!(
        gentoo_root_pre,
        NodeKind::Run {
            command: pattern![I "/bin/busybox"; I "sh"; P "utxzsh"; P "archive"],
            envs: Default::default(),
            outputs: Default::default(),
        },
        (alpine_root, root),
        (unpack_txz, "utxzsh"),
        (gentoo_dl, "archive"),
    );
    node_!(
        gentoo_root,
        NodeKind::Run {
            command: pattern![I "/bin/sh"; P "reduce"],
            envs: Default::default(),
            outputs: Default::default(),
        },
        (reduce_sh, "reduce"),
        (gentoo_root_pre, root),
    );

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

    // bootstrapping
    node_!(
        binutils_unpack,
        NodeKind::Run {
            command: pattern![I "/bin/busybox"; I "sh"; P "utxzsh"; P "archive"],
            envs: Default::default(),
            outputs: Default::default(),
        },
        (alpine_root, root),
        (unpack_txz, "utxzsh"),
        ("binutils-2.37.tar.xz", "archive"),
    );
    node_!(binutils_pass1_sh, dlocal("binutils/pass1.sh"));
    node_!(
        binutils_pass1,
        NodeKind::Run {
            command: pattern![I "/bin/sh"; P "binutils_pass1.sh"; P "binutils"; P "patch1"],
            envs: Default::default(),
            outputs: Default::default(),
        },
        (gentoo_root, root),
        (binutils_unpack, "binutils"),
        (binutils_pass1_sh, "binutils_pass1.sh"),
        ("binutils-2.37-upstream_fix-1.patch", "patch1"),
    );

    let (nodes, edges) = (g.0.node_count(), g.0.edge_count());
    let mut gsane = Graph::<()>::default();
    let _trt = gsane.take_and_merge(g, |&()| (), |&mut ()| ());
    if gsane.0.node_count() != nodes || gsane.0.edge_count() != edges {
        eprintln!("verification failed!");
        std::process::exit(1);
    }

    println!(
        "{}",
        serde_json::to_string(&gsane).expect("serialization failed")
    );
}
