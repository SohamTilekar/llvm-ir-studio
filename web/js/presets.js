/**
 * Curated LLVM IR Presets for Compiler Developers
 */

export const PRESETS = {
  collatz: {
    name: 'Collatz Loop (SSA & Phi Nodes)',
    description: 'Demonstrates basic block looping, icmp, conditional branches, and SSA phi-node merging.',
    code: `; Collatz Conjecture / 3n + 1 sequence
define i32 @collatz(i32 %n) {
entry:
  %is_one = icmp eq i32 %n, 1
  br i1 %is_one, label %done, label %loop

loop:
  %curr = phi i32 [ %n, %entry ], [ %next, %loop_next ]
  %step = phi i32 [ 0, %entry ], [ %step_inc, %loop_next ]
  %step_inc = add i32 %step, 1
  %rem = srem i32 %curr, 2
  %is_even = icmp eq i32 %rem, 0
  br i1 %is_even, label %even, label %odd

even:
  %half = sdiv i32 %curr, 2
  br label %loop_next

odd:
  %mul = mul i32 %curr, 3
  %plus1 = add i32 %mul, 1
  br label %loop_next

loop_next:
  %next = phi i32 [ %half, %even ], [ %plus1, %odd ]
  %finished = icmp eq i32 %next, 1
  br i1 %finished, label %done, label %loop

done:
  %total_steps = phi i32 [ 0, %entry ], [ %step_inc, %loop_next ]
  ret i32 %total_steps
}
`
  },

  fibonacci: {
    name: 'Fibonacci (Recursion & Multi-Return)',
    description: 'Classic recursive function showing multiple returns and base-case conditional branching.',
    code: `; Recursive Fibonacci function
define i32 @fib(i32 %n) {
entry:
  %cmp = icmp sle i32 %n, 1
  br i1 %cmp, label %base_case, label %recursive

base_case:
  ret i32 %n

recursive:
  %sub1 = sub i32 %n, 1
  %call1 = call i32 @fib(i32 %sub1)
  %sub2 = sub i32 %n, 2
  %call2 = call i32 @fib(i32 %sub2)
  %result = add i32 %call1, %call2
  ret i32 %result
}
`
  },

  switchDispatch: {
    name: 'Switch Dispatch Table (Bytecode VM)',
    description: 'Multi-target switch instruction commonly produced by state machines and interpreters.',
    code: `; Virtual machine opcode dispatcher
define i32 @dispatch_op(i32 %opcode, i32 %a, i32 %b) {
entry:
  switch i32 %opcode, label %op_unknown [
    i32 0, label %op_nop
    i32 1, label %op_add
    i32 2, label %op_sub
    i32 3, label %op_mul
    i32 4, label %op_div
  ]

op_nop:
  ret i32 %a

op_add:
  %res_add = add i32 %a, %b
  ret i32 %res_add

op_sub:
  %res_sub = sub i32 %a, %b
  ret i32 %res_sub

op_mul:
  %res_mul = mul i32 %a, %b
  ret i32 %res_mul

op_div:
  %is_zero = icmp eq i32 %b, 0
  br i1 %is_zero, label %div_by_zero, label %do_div

do_div:
  %res_div = sdiv i32 %a, %b
  ret i32 %res_div

div_by_zero:
  ret i32 -1

op_unknown:
  ret i32 -2
}
`
  },

  opaquePtr: {
    name: 'Opaque Pointers (LLVM 15+ Modern)',
    description: 'Modern LLVM IR using opaque "ptr" types, load/store, and getelementptr.',
    code: `; Modern LLVM 18+ IR with opaque pointers (ptr)
source_filename = "vector_calc.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@scale = global i32 4, align 4

define i32 @process_buffer(ptr %buf, i32 %len) {
entry:
  %valid = icmp sgt i32 %len, 0
  br i1 %valid, label %init, label %error

init:
  %factor = load i32, ptr @scale, align 4
  br label %loop

loop:
  %i = phi i32 [ 0, %init ], [ %next_i, %loop ]
  %accum = phi i32 [ 0, %init ], [ %new_accum, %loop ]
  
  ; getelementptr with opaque ptr
  %elem_ptr = getelementptr i32, ptr %buf, i32 %i
  %val = load i32, ptr %elem_ptr, align 4
  %scaled = mul i32 %val, %factor
  %new_accum = add i32 %accum, %scaled

  %next_i = add i32 %i, 1
  %done = icmp eq i32 %next_i, %len
  br i1 %done, label %exit, label %loop

exit:
  ret i32 %new_accum

error:
  ret i32 -1
}
`
  },

  typedPtrLegacy: {
    name: 'Typed Pointers (Legacy LLVM 14)',
    description: 'Legacy LLVM IR with explicit typed pointers (i32*, float*) for version compatibility testing.',
    code: `; Legacy LLVM 14 IR using typed pointers
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

define i32 @legacy_math(i32* %data, i32 %count) {
entry:
  %cmp = icmp sle i32 %count, 0
  br i1 %cmp, label %empty, label %calc

calc:
  %ptr0 = getelementptr inbounds i32, i32* %data, i64 0
  %val0 = load i32, i32* %ptr0, align 4
  %ptr1 = getelementptr inbounds i32, i32* %data, i64 1
  %val1 = load i32, i32* %ptr1, align 4
  %sum = add i32 %val0, %val1
  ret i32 %sum

empty:
  ret i32 0
}
`
  },

  vectorOps: {
    name: 'SIMD Vector Arithmetic (<4 x float>)',
    description: 'Demonstrates LLVM IR SIMD vector types and instructions (fadd, fmul, shufflevector).',
    code: `; SIMD vector operations
define <4 x float> @vector_fma(<4 x float> %a, <4 x float> %b, <4 x float> %c) {
entry:
  ; Multiply-add vector math
  %prod = fmul <4 x float> %a, %b
  %sum = fadd <4 x float> %prod, %c
  
  ; Vector shuffle (reversal)
  %rev = shufflevector <4 x float> %sum, <4 x float> undef, <4 x i32> <i32 3, i32 2, i32 1, i32 0>
  ret <4 x float> %rev
}
`
  },

  exceptionHandling: {
    name: 'Exception Handling (Invoke & LandingPad)',
    description: 'Exception handling control flow using invoke, landingpad, and resume instructions.',
    code: `; Exception handling with invoke & landingpad
declare i32 @may_throw(i32 %x)
declare i32 @__gxx_personality_v0(...)

define i32 @safe_call(i32 %arg) personality ptr @__gxx_personality_v0 {
entry:
  %res = invoke i32 @may_throw(i32 %arg)
          to label %normal_exit unwind label %catch_block

normal_exit:
  ret i32 %res

catch_block:
  %lp = landingpad { ptr, i32 }
          cleanup
          catch ptr null
  ret i32 -999
}
`
  }
};
