; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i64, i1 }*
  %tmp1 = alloca { i64, i1 }
  %s = alloca { i64, i1 }*
  %tmp2 = alloca i64*
  %x = alloca i64*
  %struct_gep = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 0
  store i64 5, i64* %struct_gep
  %struct_gep3 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep3
  %ld = load { i64, i1 }, { i64, i1 }* %tmp1
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i1 }* getelementptr ({ i64, i1 }, { i64, i1 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i1 }*
  store { i64, i1 } %ld, { i64, i1 }* %box
  store { i64, i1 }* %box, { i64, i1 }** %tmp
  %ld4 = load { i64, i1 }*, { i64, i1 }** %tmp
  store { i64, i1 }* %ld4, { i64, i1 }** %s
  %malloccall5 = tail call i8* @malloc(i32 ptrtoint (i64* getelementptr (i64, i64* null, i32 1) to i32))
  %box6 = bitcast i8* %malloccall5 to i64*
  store i64 5, i64* %box6
  store i64* %box6, i64** %tmp2
  %ld7 = load i64*, i64** %tmp2
  store i64* %ld7, i64** %x
  %deref_load = load i64*, i64** %x
  %ld8 = load i64, i64* %deref_load
  store i64 %ld8, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

declare noalias i8* @malloc(i32)
