use zakac_codegen::bc::Op;
use zakac_codegen::code::CodeBuilder;
use zakac_codegen::code::Unbound;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

#[test]
fn forward_and_backward_jumps_resolve()
{
    let mut code = CodeBuilder::new();
    let head = code.label();
    let exit = code.label();
    code.bind(head);
    code.emit(Op::Pop);
    code.jump_if_true(exit);
    code.jump(head);
    code.bind(exit);
    code.emit(Op::Ret);
    let (ops, _) = code.finish().unwrap();
    assert_eq!(ops, vec![Op::Pop, Op::JumpIfTrue(3), Op::Jump(0), Op::Ret]);
}

#[test]
fn one_label_patches_each_jump_to_it()
{
    let mut code = CodeBuilder::new();
    let end = code.label();
    code.jump_if_false(end);
    code.jump_if_true(end);
    code.jump(end);
    code.bind(end);
    code.emit(Op::Ret);
    let (ops, _) = code.finish().unwrap();
    assert_eq!(ops, vec![Op::JumpIfFalse(3), Op::JumpIfTrue(3), Op::Jump(3), Op::Ret]);
}

#[test]
fn a_label_can_mark_the_end()
{
    let mut code = CodeBuilder::new();
    let end = code.label();
    code.jump(end);
    code.bind(end);
    assert_eq!(code.pc(), 1);
    assert_eq!(code.finish().unwrap().0, vec![Op::Jump(1)]);
}

#[test]
fn unbound_labels_are_an_error()
{
    let mut code = CodeBuilder::new();
    let _bound = code.label();
    let lost = code.label();
    code.jump(lost);
    assert_eq!(code.finish(), Err(Unbound(1)));
}

#[test]
fn spans_follow_the_cursor()
{
    let (a, b) =
        (Span::new(Pos::new(2), Pos::new(5)), Span::new(Pos::new(7), Pos::new(9)));
    let mut code = CodeBuilder::new();
    let end = code.label();
    assert_eq!(code.at(a), Span::PLACEHOLDER);
    code.emit(Op::Pop);
    assert_eq!(code.at(b), a);
    code.jump(end);
    code.bind(end);
    let (ops, spans) = code.finish().unwrap();
    assert_eq!(ops, vec![Op::Pop, Op::Jump(2)]);
    assert_eq!(spans, vec![a, b]);
}
