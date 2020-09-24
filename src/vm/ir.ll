; ModuleID = 'main'
source_filename = "main"

define { i8, <{ i64 }> } @Some(i64 %0) {
basic_block0:
  %retvar = alloca { i8, <{ i64 }> }
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar, i32 0, i32 0
  store i8 0, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds <{ i64 }>, <{ i64 }>* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load { i8, <{ i64 }> }, { i8, <{ i64 }> }* %retvar
  ret { i8, <{ i64 }> } %load_ret
}

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
