use amml_data::{OperandKind, Step};
use amml_derive::StepTrace;

#[test]
fn describe_prints_operand_kinds_as_recipes_spell_them() {
    let cases = [
        (OperandKind::Displacement, "operand 1 displacement"),
        (OperandKind::Immediate, "operand 1 immediate"),
        (OperandKind::RipTarget, "operand 1 rip_target"),
        (OperandKind::BranchTarget, "operand 1 branch_target"),
    ];
    for (kind, text) in cases {
        assert_eq!(StepTrace::describe(&Step::Operand { index: 1, kind }), text);
    }
}
