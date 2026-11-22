; LLVM 14 IR with typed pointers (i32*, etc.)
source_filename = "valid_typed.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

define i32 @legacy_calculate(i32* %data, i32 %n) {
entry:
  %cmp = icmp sle i32 %n, 0
  br i1 %cmp, label %zero, label %compute

zero:
  ret i32 0

compute:
  %val = load i32, i32* %data, align 4
  %res = mul i32 %val, 2
  ret i32 %res
}
