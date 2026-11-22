; Branching and looping LLVM IR for CFG testing
define i32 @collatz(i32 %n) {
entry:
  %is_one = icmp eq i32 %n, 1
  br i1 %is_one, label %done, label %loop

loop:
  %curr = phi i32 [ %n, %entry ], [ %next, %loop_next ]
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
  ret i32 1
}
