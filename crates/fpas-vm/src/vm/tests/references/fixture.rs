//! Bytecode invocations with caller storage retained outside the worker.

use super::*;

pub(super) fn invocation(
    root: Vec<Instruction>,
    callees: Vec<(Vec<Instruction>, u16, Vec<u8>)>,
    registers: u16,
) -> Worker {
    let mut image = super::super::unverified(
        root,
        vec![Constant::Integer(9)],
        vec!["root", "reference-call.fpas"],
        registers,
    );
    for (index, (code, registers, modes)) in callees.into_iter().enumerate() {
        let start = image.code.len() as u32;
        let arity = modes.last().map_or(0, |position| position + 1);
        image.code.extend(code);
        let mut info = image.functions[0].clone();
        let mut strings: Vec<String> = image.strings.iter().map(str::to_owned).collect();
        info.name = fpas_bytecode::StringId::new(strings.len() as u32);
        strings.push(format!("callee{}", index + 1));
        image.strings = fpas_bytecode::StringTable::new(strings);
        info.code = CodeRange::new(
            InstructionAddress::new(start),
            InstructionAddress::new(image.code.len() as u32),
        );
        info.arity = arity;
        info.var_parameters = modes;
        info.register_count = registers;
        image.functions.push(info);
        image.source_map.runs.push(fpas_bytecode::SourceRun {
            instruction_start: InstructionAddress::new(start),
            source: fpas_bytecode::SourceId::new(0),
            line: 42 + index as u32,
            column: 1,
        });
    }
    Worker::new(Arc::new(image.verify().unwrap())).unwrap()
}

pub(super) fn storage(worker: &mut Worker, value: Value) -> Arc<Mutex<Value>> {
    let cell = Arc::new(Mutex::new(value));
    worker
        .store_register(0, Value::Cell(Arc::clone(&cell)))
        .unwrap();
    cell
}

pub(super) fn set_nine() -> Vec<Instruction> {
    vec![
        abx(Opcode::LoadConstant, 1, 0),
        abc(Opcode::WriteReference, 0, 1, 0),
        return_unit(),
    ]
}

pub(super) fn run(worker: &mut Worker) -> Result<(), crate::vm::VmError> {
    worker.run_in_place().map(|_| ())
}
