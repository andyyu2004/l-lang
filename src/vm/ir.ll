; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca i64*
  %x = alloca i64*
  %tmp1 = alloca i64*
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i64* getelementptr (i64, i64* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to i64*
  store i64* %box, i64** %tmp
  %ld = load i64*, i64** %tmp
  store i64* %ld, i64** %x
  %ld2 = load i64*, i64** %x
  store i64* %ld2, i64** %tmp1
  store i64 5, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

declare noalias i8* @malloc(i32)
