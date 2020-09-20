; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca i64*
  %tmp1 = alloca i64
  %ptr = alloca i64*
  store i64 5, i64* %tmp1
  store i64* %tmp1, i64** %tmp
  %load = load i64*, i64** %tmp
  store i64* %load, i64** %ptr
  %deref_load = load i64*, i64** %ptr
  store i64 19, i64* %deref_load
  %deref_load2 = load i64*, i64** %ptr
  %load3 = load i64, i64* %deref_load2
  store i64 %load3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
