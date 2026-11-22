use ir_version::LlvmVersion;

#[derive(Debug, Clone)]
pub struct InstructionDoc {
    pub name: &'static str,
    pub category: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
    pub operands: &'static str,
    pub version_notes: Option<&'static str>,
}

impl InstructionDoc {
    pub fn to_markdown(&self, version: LlvmVersion) -> String {
        let mut md = format!(
            "### LLVM IR: `{}` ({})\n\n**Syntax:**\n```llvm\n{}\n```\n\n**Description:**\n{}\n\n**Operands:**\n{}\n",
            self.name, self.category, self.syntax, self.description, self.operands
        );

        if let Some(notes) = self.version_notes {
            md.push_str(&format!("\n> **Version Notes (Active: {}):**\n> {}\n", version, notes));
        }

        md
    }
}

pub fn get_instruction_doc(name: &str) -> Option<InstructionDoc> {
    match name {
        // --- Terminators ---
        "ret" => Some(InstructionDoc {
            name: "ret",
            category: "Terminator",
            syntax: "ret <type> <value>\nret void",
            description: "The 'ret' instruction is used to return control flow (and optionally a value) from a function back to the caller.",
            operands: "Optional return value of first-class type, or 'void'.",
            version_notes: Some("Standard terminator in all LLVM versions."),
        }),
        "br" => Some(InstructionDoc {
            name: "br",
            category: "Terminator",
            syntax: "br label <dest>\nbr i1 <cond>, label <iftrue>, label <iffalse>",
            description: "The 'br' instruction is used to cause control flow to transfer to a different basic block in the current function. Can be unconditional or conditional.",
            operands: "Conditional branch takes an `i1` condition and two basic block labels. Unconditional takes one destination label.",
            version_notes: None,
        }),
        "switch" => Some(InstructionDoc {
            name: "switch",
            category: "Terminator",
            syntax: "switch <intty> <value>, label <defaultdest> [ <intty> <val>, label <dest> ... ]",
            description: "The 'switch' instruction is used to transfer control flow to one of several different places based on an integer value.",
            operands: "Comparison integer value, default destination label, and a table of constant-dest pairs.",
            version_notes: None,
        }),
        "indirectbr" => Some(InstructionDoc {
            name: "indirectbr",
            category: "Terminator",
            syntax: "indirectbr <type> <address>, [ label <dest1>, label <dest2>, ... ]",
            description: "The 'indirectbr' instruction implements an indirect branch to a label within the current function, whose address was taken by 'blockaddress'.",
            operands: "Address of type ptr (or i8* pre-15) and list of possible destinations.",
            version_notes: Some("Pointer type is `ptr` in modern LLVM; typed pointer `i8*` pre-15."),
        }),
        "invoke" => Some(InstructionDoc {
            name: "invoke",
            category: "Terminator",
            syntax: "<result> = invoke [cconv] [ret attrs] <ty>|<fnty> <fnptrval>(<function args>) [fn attrs]\n              [operand bundles] to label <normal label> unwind label <exception label>",
            description: "The 'invoke' instruction causes control to transfer to a specified function, with the possibility of handling exceptions via the unwind branch.",
            operands: "Callee, arguments, normal return label, and exception landingpad label.",
            version_notes: None,
        }),
        "resume" => Some(InstructionDoc {
            name: "resume",
            category: "Terminator",
            syntax: "resume <type> <value>",
            description: "The 'resume' instruction resumes propagation of an existing (in-flight) exception whose unwinding was interrupted with a landingpad instruction.",
            operands: "Exception object value (typically struct `{ ptr, i32 }`).",
            version_notes: None,
        }),
        "unreachable" => Some(InstructionDoc {
            name: "unreachable",
            category: "Terminator",
            syntax: "unreachable",
            description: "The 'unreachable' instruction tells the optimizer that control flow will never reach this point. It has undefined behavior if executed.",
            operands: "None.",
            version_notes: None,
        }),

        // --- Memory Access ---
        "alloca" => Some(InstructionDoc {
            name: "alloca",
            category: "Memory Access",
            syntax: "<result> = alloca [inalloca] <type> [, <ty> <NumElements>] [, align <alignment>] [, addrspace(<num>)]",
            description: "The 'alloca' instruction allocates memory on the stack frame of the currently executing function, to be automatically released when this function returns.",
            operands: "Type to allocate, optional element count, optional alignment, and optional address space.",
            version_notes: Some("Returns `ptr` in LLVM 15+ (or `<type>*` pre-15)."),
        }),
        "load" => Some(InstructionDoc {
            name: "load",
            category: "Memory Access",
            syntax: "<result> = load [volatile] <ty>, ptr <pointer> [, align <alignment>]\n(Legacy pre-15: <result> = load <ty>, <ty>* <pointer>)",
            description: "The 'load' instruction is used to read from memory.",
            operands: "Type to load, pointer operand, optional alignment, optional metadata.",
            version_notes: Some("In LLVM 15+, pointer operand is opaque `ptr` and the explicit loaded type `<ty>` must precede it. Typed pointers (`<ty>*`) are deprecated/removed in LLVM 17+."),
        }),
        "store" => Some(InstructionDoc {
            name: "store",
            category: "Memory Access",
            syntax: "store [volatile] <ty> <value>, ptr <pointer> [, align <alignment>]\n(Legacy pre-15: store <ty> <value>, <ty>* <pointer>)",
            description: "The 'store' instruction is used to write to memory.",
            operands: "Value to store (with type) and destination pointer operand.",
            version_notes: Some("Requires `ptr` pointer in LLVM 17+. Value type is always explicit."),
        }),
        "getelementptr" => Some(InstructionDoc {
            name: "getelementptr",
            category: "Memory Access",
            syntax: "<result> = getelementptr [inbounds] <ty>, ptr <ptrval>{, <ty> <idx>}*\n(Legacy: getelementptr <pty>, <pty>* <ptrval>{, <ty> <idx>}*)",
            description: "The 'getelementptr' instruction calculates the address of a subelement of an aggregate data structure. It only performs address calculation and does not access memory.",
            operands: "Source pointee type, base pointer, and list of indices.",
            version_notes: Some("Since LLVM 15+, the pointee type `<ty>` must be explicitly specified because `ptr` is opaque. In LLVM 17+, typed pointer syntax is disallowed."),
        }),
        "fence" => Some(InstructionDoc {
            name: "fence",
            category: "Memory Access",
            syntax: "fence [syncscope(\"<target-scope>\")] <ordering>",
            description: "The 'fence' instruction is used to introduce happens-before constraints between operations across threads.",
            operands: "Atomic ordering (e.g. acquire, release, seq_cst) and optional synchronization scope.",
            version_notes: None,
        }),
        "cmpxchg" => Some(InstructionDoc {
            name: "cmpxchg",
            category: "Memory Access",
            syntax: "cmpxchg [weak] [volatile] ptr <pointer>, <ty> <cmp>, <ty> <new> [syncscope(\"<target-scope>\")] <success ordering> <failure ordering>",
            description: "The 'cmpxchg' instruction atomically compares value in memory with a comparison value, and if equal writes a new value.",
            operands: "Pointer, compare value, new value, memory orderings.",
            version_notes: Some("Returns `{ <ty>, i1 }` struct indicating old value and whether exchange succeeded."),
        }),
        "atomicrmw" => Some(InstructionDoc {
            name: "atomicrmw",
            category: "Memory Access",
            syntax: "atomicrmw [volatile] <operation> ptr <pointer>, <ty> <value> [syncscope(\"<target-scope>\")] <ordering>",
            description: "The 'atomicrmw' instruction atomically modifies memory using an operation like add, sub, and, xchg, etc.",
            operands: "Atomic operation mnemonic, pointer, value operand, memory ordering.",
            version_notes: None,
        }),

        // --- Standard Operations ---
        "add" | "fadd" => Some(InstructionDoc {
            name: "add / fadd",
            category: "Binary Arithmetic",
            syntax: "<result> = add [nuw] [nsw] <ty> <op1>, <op2>\n<result> = fadd [fast-math flags]* <ty> <op1>, <op2>",
            description: "Adds two integer, vector, or floating-point operands of identical type.",
            operands: "Two operands of matching type.",
            version_notes: None,
        }),
        "sub" | "fsub" => Some(InstructionDoc {
            name: "sub / fsub",
            category: "Binary Arithmetic",
            syntax: "<result> = sub [nuw] [nsw] <ty> <op1>, <op2>\n<result> = fsub [fast-math flags]* <ty> <op1>, <op2>",
            description: "Subtracts operand 2 from operand 1.",
            operands: "Two operands of matching type.",
            version_notes: None,
        }),
        "mul" | "fmul" => Some(InstructionDoc {
            name: "mul / fmul",
            category: "Binary Arithmetic",
            syntax: "<result> = mul [nuw] [nsw] <ty> <op1>, <op2>\n<result> = fmul [fast-math flags]* <ty> <op1>, <op2>",
            description: "Multiplies two operands.",
            operands: "Two operands of matching type.",
            version_notes: None,
        }),
        "udiv" | "sdiv" | "fdiv" => Some(InstructionDoc {
            name: "div",
            category: "Binary Arithmetic",
            syntax: "<result> = [u|s|f]div [exact] <ty> <op1>, <op2>",
            description: "Divides operand 1 by operand 2 (unsigned, signed, or float). Division by zero yields undefined behavior.",
            operands: "Two operands of matching type.",
            version_notes: None,
        }),
        "icmp" => Some(InstructionDoc {
            name: "icmp",
            category: "Comparison",
            syntax: "<result> = icmp <cond> <ty> <op1>, <op2>",
            description: "The 'icmp' instruction returns a boolean value or vector of boolean values based on comparison of two integer, pointer, or vector operands.\nConditions: `eq`, `ne`, `ugt`, `uge`, `ult`, `ule`, `sgt`, `sge`, `slt`, `sle`.",
            operands: "Condition flag, operand type, and two operands.",
            version_notes: None,
        }),
        "fcmp" => Some(InstructionDoc {
            name: "fcmp",
            category: "Comparison",
            syntax: "<result> = fcmp [fast-math flags]* <cond> <ty> <op1>, <op2>",
            description: "The 'fcmp' instruction compares two floating-point values according to condition (e.g. `oeq`, `une`, `olt`, etc.).",
            operands: "Condition flag, float type, and two operands.",
            version_notes: None,
        }),
        "phi" => Some(InstructionDoc {
            name: "phi",
            category: "SSA Merge",
            syntax: "<result> = phi <ty> [ <val0>, <label0>], [ <val1>, <label1> ], ...",
            description: "The 'phi' instruction is used to implement the φ node in the SSA graph. At runtime, it receives the value corresponding to the basic block from which execution branched into the current block.",
            operands: "Pairs of value and incoming basic block label for all predecessor blocks.",
            version_notes: Some("All phi instructions must appear at the very beginning of a basic block before any non-phi instructions."),
        }),
        "select" => Some(InstructionDoc {
            name: "select",
            category: "Control / Value Selection",
            syntax: "<result> = select <selty> <cond>, <ty> <val1>, <ty> <val2>",
            description: "The 'select' instruction is used to choose one value based on a condition, without branching.",
            operands: "`i1` or `<N x i1>` condition, true value, false value.",
            version_notes: None,
        }),
        "call" => Some(InstructionDoc {
            name: "call",
            category: "Function Call",
            syntax: "<result> = [tail | musttail | notail] call [fast-math flags] [cconv] [ret attrs]\n              <ty>|<fnty> <fnptrval>(<function args>) [fn attrs] [operand bundles]",
            description: "The 'call' instruction is used to cause control flow to transfer to a specified function, with incoming arguments.",
            operands: "Return type, function pointer/name, and argument list with types.",
            version_notes: Some("Direct calls use `@fn_name`; indirect calls take a pointer operand."),
        }),
        "bitcast" => Some(InstructionDoc {
            name: "bitcast",
            category: "Conversion",
            syntax: "<result> = bitcast <ty> <value> to <ty2>",
            description: "The 'bitcast' instruction converts value to type ty2 without changing any bits. Requires identical bit sizes.",
            operands: "Source type & value, destination type.",
            version_notes: Some("With opaque pointers (`ptr`), bitcasting between different pointer types is completely obsolete and unnecessary."),
        }),
        "trunc" | "zext" | "sext" => Some(InstructionDoc {
            name: "trunc / zext / sext",
            category: "Integer Conversion",
            syntax: "<result> = trunc <ty> <value> to <ty2>\n<result> = zext <ty> <value> to <ty2>\n<result> = sext <ty> <value> to <ty2>",
            description: "Truncates, zero-extends, or sign-extends an integer or vector of integers to another integer bit width.",
            operands: "Source value and target integer type.",
            version_notes: None,
        }),
        "ptrtoint" | "inttoptr" => Some(InstructionDoc {
            name: "ptrtoint / inttoptr",
            category: "Pointer Conversion",
            syntax: "<result> = ptrtoint <ty> <value> to <ty2>\n<result> = inttoptr <ty> <value> to <ty2>",
            description: "Converts a pointer to an integer of specified width, or an integer to a pointer.",
            operands: "Source pointer/int, target int/pointer.",
            version_notes: None,
        }),
        "freeze" => Some(InstructionDoc {
            name: "freeze",
            category: "Other",
            syntax: "<result> = freeze <ty> <value>",
            description: "The 'freeze' instruction stops propagation of undef and poison values. If operand is poison/undef, it returns an arbitrary fixed value of that type.",
            operands: "Single value of first class type.",
            version_notes: Some("Introduced in LLVM 10+ to fix miscompilations around undef/poison."),
        }),
        _ => None,
    }
}
