use zakac_explorer::compile::Options;
use zakac_explorer::compile::compile;
use zakac_explorer::compile::graph::Graph;
use zakac_explorer::compile::graph::MARGIN;
use zakac_explorer::compile::graph::MAX_ZOOM;
use zakac_explorer::compile::graph::MIN_ZOOM;
use zakac_explorer::compile::graph::NODE_HEIGHT;
use zakac_explorer::compile::graph::View;
use zakac_explorer::compile::graph::graph;
use zakac_explorer::compile::samples::SAMPLES;
use zakac_explorer::compile::tree::TreeNode;

/// Number of nodes in a tree.
fn count(node: &TreeNode) -> usize
{
    1 + node.children.iter().map(count).sum::<usize>()
}

/// The graphs of all samples.
fn sample_graphs() -> Vec<(&'static str, TreeNode, Graph)>
{
    let tree = |text: &str| compile(text, &Options::default()).tree;
    SAMPLES.iter().map(|s| (s.name, tree(s.text), graph(&tree(s.text)))).collect()
}

#[test]
fn all_tree_nodes_are_placed_once()
{
    for (name, tree, g) in sample_graphs()
    {
        assert_eq!(g.nodes.len(), count(&tree), "{name}");
        assert!(g.nodes[0].parent.is_none(), "{name}");
        assert!(g.nodes.iter().skip(1).all(|n| n.parent.is_some()), "{name}");
    }
}

#[test]
fn children_sit_a_row_under_their_parent()
{
    for (name, _, g) in sample_graphs()
    {
        for node in &g.nodes
        {
            let Some(parent) = node.parent.map(|p| &g.nodes[p])
            else
            {
                continue;
            };
            assert!(node.y > parent.y + NODE_HEIGHT, "{name}");
        }
    }
}

#[test]
fn nodes_of_a_row_do_not_overlap()
{
    for (name, _, g) in sample_graphs()
    {
        let mut rows: Vec<Vec<(f64, f64)>> = Vec::new();
        for node in &g.nodes
        {
            let row = ((node.y - MARGIN) / NODE_HEIGHT) as usize;
            rows.resize(rows.len().max(row + 1), Vec::new());
            rows[row].push((node.x, node.x + node.width));
        }
        for mut row in rows
        {
            row.sort_by(|a, b| a.0.total_cmp(&b.0));
            assert!(row.windows(2).all(|w| w[0].1 <= w[1].0), "{name}");
        }
    }
}

#[test]
fn the_graph_holds_all_its_nodes()
{
    for (name, _, g) in sample_graphs()
    {
        for node in &g.nodes
        {
            assert!(
                node.x >= MARGIN - 0.001 && node.x + node.width <= g.width,
                "{name}"
            );
            assert!(node.y + NODE_HEIGHT <= g.height, "{name}");
        }
    }
}

#[test]
fn a_lone_parent_is_centered_over_its_child()
{
    let out = compile("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။", &Options::default());
    let g = graph(&out.tree);
    let root = &g.nodes[0];
    let decl = &g.nodes[1];
    assert!((root.center() - decl.center()).abs() < 0.001);
    let int = g.nodes.iter().find(|n| n.label == "int").unwrap();
    assert!(int.literal);
    assert_eq!(int.value, "၁၈");
}

#[test]
fn zooming_holds_the_point_under_the_cursor_still()
{
    let view = View::default().moved(40.0, 20.0);
    let zoomed = view.zoom_at(2.0, 100.0, 60.0);
    assert_eq!(zoomed.zoom, 2.0);
    // The graph point under (100, 60) is the same before and after.
    let before = ((100.0 - view.x) / view.zoom, (60.0 - view.y) / view.zoom);
    let after = ((100.0 - zoomed.x) / zoomed.zoom, (60.0 - zoomed.y) / zoomed.zoom);
    assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
}

#[test]
fn zoom_stays_in_its_bounds()
{
    assert_eq!(View::default().zoom_at(100.0, 0.0, 0.0).zoom, MAX_ZOOM);
    assert_eq!(View::default().zoom_at(0.001, 0.0, 0.0).zoom, MIN_ZOOM);
}

#[test]
fn fitting_shrinks_a_big_graph_and_centers_a_small_one()
{
    let big = Graph { width: 2000.0, height: 500.0, ..Graph::default() };
    let fit = View::fit(&big, 1000.0, 800.0);
    assert_eq!(fit.zoom, 0.5);
    assert_eq!(fit.x, 0.0);
    let small = Graph { width: 200.0, height: 100.0, ..Graph::default() };
    let fit = View::fit(&small, 1000.0, 800.0);
    assert_eq!(fit.zoom, 1.0);
    assert_eq!((fit.x, fit.y), (400.0, 350.0));
}

#[test]
fn a_big_graph_opens_on_its_root()
{
    let sample = SAMPLES.iter().find(|s| s.name == "fizzbuzz").unwrap();
    let g = graph(&compile(sample.text, &Options::default()).tree);
    assert!(g.width > 800.0);
    let view = View::focus_root(&g, 800.0, 600.0);
    assert_eq!(view.zoom, 1.0);
    assert_eq!(view.y, 0.0);
    assert!((view.x + g.nodes[0].center() - 400.0).abs() < 1e-9);
    let roomy = View::focus_root(&g, g.width, g.height);
    assert_eq!(roomy, View::fit(&g, g.width, g.height));
}
