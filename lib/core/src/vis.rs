use petgraph::graph::NodeIndex;

use crate::control::ControlGraph;

pub fn visualize_graph(cg: &ControlGraph) -> String {
    let node_indexes = cg.get_node_indexes();

    let mut src = vec![];

    // declare the node labels
    for (node_num, &node_index) in node_indexes.iter().enumerate() {
        let node = cg.get_node(node_index);
        let ident = node.get_ident();

        let root_label = format!(
            "{}{}",
            match ident {
                "ContainerInput" | "ContainerOutput" => "",
                _ => "<o>",
            },
            match ident {
                "Constant" => format!("{node:?}")
                    .trim_start_matches("Const(Sample(")
                    .trim_end_matches("))")
                    .trim_end_matches(".0")
                    .to_string(),
                "ContainerInput" | "ContainerOutput" =>
                    node.get_input_labels()[0].clone().into_owned(),
                _ => ident.into(),
            }
        );

        let input_labels = node.get_input_labels();

        let fmt = if !input_labels.is_empty()
            && ident != "ContainerInput"
            && ident != "ContainerOutput"
        {
            let mut input_labels_with_ids = vec![];

            for (label_num, input_label) in input_labels.iter().enumerate() {
                input_labels_with_ids.push(format!("<i{}>{}", label_num, input_label));
            }

            format!("{{{}}}|{root_label}", input_labels_with_ids.join("|"))
        } else {
            root_label
        };

        if ident == "ContainerInput" {
            src.push(format!(
                "s{node_num} [label = \"{{{fmt}}}\";style=filled;color=deepskyblue;];"
            ));
        } else if ident == "ContainerOutput" {
            src.push(format!(
                "s{node_num} [label = \"{{{fmt}}}\";style=filled;color=coral;];"
            ));
        } else if ident == "Constant" {
            src.push(format!("s{node_num} [label = \"{{{fmt}}}\";width = 0;height = 0;style = \"rounded,filled\";color = lightgray;fontname=\"MonoLisa\";];"));
        } else {
            src.push(format!("s{node_num} [label = \"{{{fmt}}}\";];"));
        }
    }

    let (_, rendered) = subgraph(cg, 0);

    format!(
        r#"digraph structs {{
        node [shape = record;];
        edge [arrowhead = none;];
        rankdir = LR;
        nodesep = .05;

        aout;
        {}
        {rendered}
    }}"#,
        src.join("\n"),
    )
}

#[derive(Default)]
struct SubgraphCalc {
    ignore_all: Vec<NodeIndex>,
    ignore_in: Vec<NodeIndex>,
    ignore_out: Vec<NodeIndex>,
}

fn subgraph(cg: &ControlGraph, i: usize) -> (SubgraphCalc, String) {
    let children = &cg.get_container_children()[i];

    let mut rendered = vec![];
    let mut calc = SubgraphCalc::default();

    for &c in children {
        let subgraph_rendered = subgraph(cg, c + 1);
        rendered.push(subgraph_rendered.1);

        calc.ignore_all = [calc.ignore_all, subgraph_rendered.0.ignore_all].concat();
        calc.ignore_in = [calc.ignore_in, subgraph_rendered.0.ignore_in].concat();
        calc.ignore_out = [calc.ignore_out, subgraph_rendered.0.ignore_out].concat();
    }

    calc.ignore_all.dedup();
    calc.ignore_in.dedup();
    calc.ignore_out.dedup();

    let node_indexes = cg.get_node_indexes();
    let container_node_indexes = if i == 0 {
        cg.get_node_indexes()
    } else {
        cg.get_container_member_indexes(i - 1)
    };

    // declare the edges
    for (node_num, &node_index) in node_indexes.iter().enumerate() {
        if !container_node_indexes.contains(&node_index) {
            continue;
        }
        if calc.ignore_all.contains(&node_index) || calc.ignore_out.contains(&node_index) {
            continue;
        }

        let node = cg.get_node(node_index);

        let node_ident = node.get_ident();
        let children = cg.get_node_children(node_index);

        for (child_edge, child_node_index) in children {
            if !(container_node_indexes.contains(&child_node_index)
                || i == 0 && child_node_index == NodeIndex::new(0))
            {
                continue;
            }
            if calc.ignore_all.contains(&child_node_index)
                || calc.ignore_in.contains(&child_node_index)
            {
                continue;
            }

            rendered.push(
                match node_indexes
                    .iter()
                    .enumerate()
                    .find(|(_, &idx)| idx == child_node_index)
                {
                    Some((child_node_id, &child_node_index)) => {
                        let child_node = cg.get_node(child_node_index);
                        let ident = child_node.get_ident();
                        if node_ident == "ContainerOutput" && ident == "ContainerInput" {
                            format!("s{node_num}:e -> s{child_node_id}:w;")
                        } else if ident == "ContainerInput" || ident == "ContainerOutput" {
                            format!("s{node_num}:o -> s{child_node_id}:w;")
                        } else if node_ident == "ContainerInput" || node_ident == "ContainerOutput"
                        {
                            //rustfmt what are u doing to that opening bracket
                            format!("s{node_num}:e -> s{child_node_id}:i{child_edge};")
                        } else {
                            format!("s{node_num}:o -> s{child_node_id}:i{child_edge};")
                        }
                    }
                    None => {
                        if node_ident == "ContainerInput" || node_ident == "ContainerOutput" {
                            format!("s{node_num}:e -> aout;")
                        } else {
                            format!("s{node_num}:o -> aout;")
                        }
                    }
                },
            );
        }
    }

    // add calculated nodes to calc
    for &node_index in container_node_indexes {
        let node = cg.get_node(node_index);
        match node.get_ident() {
            "ContainerInput" => calc.ignore_out.push(node_index),
            "ContainerOutput" => calc.ignore_in.push(node_index),
            _ => calc.ignore_all.push(node_index),
        }
    }

    if i == 0 {
        (calc, rendered.join("\n"))
    } else {
        (
            calc,
            format!(
                "subgraph cluster_{i} {{\n{}\nlabel = \"{}\";\nstyle = \"dashed\";}}",
                rendered.join("\n"),
                cg.get_container_ident(i - 1)
            ),
        )
    }
}
