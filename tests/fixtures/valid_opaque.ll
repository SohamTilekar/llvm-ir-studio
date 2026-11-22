; LLVM 18+ IR with opaque pointers (ptr)
source_filename = "valid_opaque.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@global_counter = global i32 0, align 4

define i32 @calculate(ptr %data, i32 %n) {
entry:
  %cmp = icmp sle i32 %n, 0
  br i1 %cmp, label %zero, label %compute

zero:
  ret i32 0

compute:
  %val = load i32, ptr %data, align 4
  %res = mul i32 %val, 2
  ret i32 %res
}
