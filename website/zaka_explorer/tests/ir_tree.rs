use zaka_explorer::compile::Options;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::tree::TreeNode;

/// All nodes of the tree, parents before children.
fn flatten(node: &TreeNode) -> Vec<&TreeNode>
{
    std::iter::once(node).chain(node.children.iter().flat_map(flatten)).collect()
}

const TEXT: &str = "ကိန်းသေ အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။
အသက် + ၂ ကို ပြသပါ။";

#[test]
fn module_holds_globals_and_main()
{
    let out = compile(TEXT, &Options::default());
    let labels: Vec<&str> = out.ir_plain.children.iter().map(|c| c.label).collect();
    assert_eq!(
        (out.ir_plain.label, labels.as_slice()),
        ("module", ["globals", "main"].as_slice())
    );
    let slot = &out.ir_plain.children[0].children[0];
    assert_eq!((slot.label, slot.detail.as_str()), ("slot", "g0 အသက် const"));
    assert_eq!(slot.kind.as_deref(), Some("isize"));
}

#[test]
fn reads_name_their_slot_and_constants_show_their_value()
{
    let out = compile(TEXT, &Options::default());
    let plain = flatten(&out.ir_plain);
    let read = plain.iter().find(|n| n.label == "read").expect("a read");
    assert_eq!(read.detail, "g0 အသက်");
    assert_eq!(&TEXT[read.range.lo as usize..read.range.hi as usize], "အသက်");
    let folded = flatten(&out.ir_folded);
    assert!(folded.iter().all(|n| n.label != "read"));
    let sum = folded.iter().rev().find(|n| n.label == "const").expect("a constant");
    assert_eq!(sum.detail, "20:isize");
    assert!(sum.is_literal());
}

#[test]
fn trees_are_blank_when_lowering_is_skipped()
{
    let out = compile("က ကို ပြသပါ။", &Options::default());
    assert!(!out.lowered);
    assert!(out.ir_plain.children.is_empty() && out.ir_folded_sexpr.is_empty());
}
