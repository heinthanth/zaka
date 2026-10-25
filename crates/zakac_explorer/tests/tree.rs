use zakac_explorer::compile::Options;
use zakac_explorer::compile::compile;
use zakac_explorer::compile::tree::TreeNode;

/// All nodes of the tree, parents before children.
fn flatten(node: &TreeNode) -> Vec<&TreeNode>
{
    std::iter::once(node).chain(node.children.iter().flat_map(flatten)).collect()
}

#[test]
fn decl_has_name_kind_and_value()
{
    let out = compile("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။", &Options::default());
    assert_eq!(out.tree.label, "program");
    let decl = &out.tree.children[0];
    assert_eq!(decl.label, "decl");
    let labels: Vec<&str> = decl.children.iter().map(|c| c.label).collect();
    assert_eq!(labels, ["ident", "kind", "int"]);
    let int = &decl.children[2];
    assert!(int.id.is_some());
    assert_eq!(int.kind.as_deref(), Some("isize"));
    assert_eq!(decl.children[0].detail, "အသက်");
}

#[test]
fn kinds_are_absent_when_the_check_is_skipped()
{
    let out = compile("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။ ;", &Options::default());
    assert!(!out.checked);
    assert!(flatten(&out.tree).iter().all(|n| n.kind.is_none()));
}

#[test]
fn node_ranges_nest()
{
    let text = "အကယ်၍ ၁ သည် ၂ ထက် ကြီးလျှင် \"ok\" ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    for node in flatten(&out.tree).iter().skip(1)
    {
        for child in &node.children
        {
            assert!(node.range.lo <= child.range.lo && child.range.hi <= node.range.hi);
        }
    }
}
