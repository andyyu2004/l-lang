; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca i64*
  %x = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i64* getelementptr (i64, i64* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to i64*
  store i64 5, i64* %box
  store i64* %box, i64** %tmp
  %load = load i64*, i64** %tmp
  store i64* %load, i64** %x
  %deref_load = load i64*, i64** %x
  %load1 = load i64, i64* %deref_load
  store i64 %load1, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block1:                                     ; No predecessors!
  %load_ret2 = load i64, i64* %retvar
  ret i64 %load_ret2
}

declare noalias i8* @malloc(i32)
