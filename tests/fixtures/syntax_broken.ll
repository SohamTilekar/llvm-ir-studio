; Syntax-broken LLVM IR for error recovery testing
define i32 @broken_fn(i32 %x {
  %val = add i32 %x,
  ret i32
