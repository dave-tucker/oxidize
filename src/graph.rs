use daggy::{Dag, NodeIndex};
use std::collections::HashMap;

use crate::types::{Makefile, Rule};

pub fn from_makefile<'a>(makefile: Makefile) -> Result<Dag<&str, u32, u32>, &'a str> {
    let mut res = Dag::new();
    let mut nodes: HashMap<&str, NodeIndex> = HashMap::new();

    for i in makefile.rules {
        for t in i.targets {
            if t == ".PHONY" {
                continue;
            }

            if !nodes.contains_key(t) {
                let x = res.add_node(t);
                nodes.insert(t, x);
            }

            for p in &i.prerequsities {
                if !nodes.contains_key(p) {
                    let x = res.add_node(p);
                    nodes.insert(p, x);
                }
                let tn = nodes.get(t).unwrap();
                let pn = nodes.get(p).unwrap();

                if res.add_edge(*tn, *pn, 1).is_err() {
                    return Err("Graph would cycle");
                }
            }
        }
    }
    Ok(res)
}

#[test]
fn test_from_makefile() {
    let _d = from_makefile(Makefile {
        variables: Vec::new(),
        rules: vec![Rule {
            targets: vec!["foo", "bar"],
            prerequsities: vec!["baz", "quux"],
            recipe: Vec::new(),
        }],
    })
    .unwrap();
}
