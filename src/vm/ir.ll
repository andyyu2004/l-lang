; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  store i64 5, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
