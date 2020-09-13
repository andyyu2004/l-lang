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
  %tmp3 = alloca {}
  %struct_gep = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 0
  store i64 5, i64* %struct_gep
  %struct_gep4 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep4
  %load = load { i64, i1 }, { i64, i1 }* %tmp1
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i1 }* getelementptr ({ i64, i1 }, { i64, i1 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i1 }*
  store { i64, i1 } %load, { i64, i1 }* %box
  store { i64, i1 }* %box, { i64, i1 }** %tmp
  %load5 = load { i64, i1 }*, { i64, i1 }** %tmp
  store { i64, i1 }* %load5, { i64, i1 }** %s
  %malloccall6 = tail call i8* @malloc(i32 ptrtoint (i64* getelementptr (i64, i64* null, i32 1) to i32))
  %box7 = bitcast i8* %malloccall6 to i64*
  store i64 5, i64* %box7
  store i64* %box7, i64** %tmp2
  %load8 = load i64*, i64** %tmp2
  store i64* %load8, i64** %x
  %load9 = load i64*, i64** %x
  %fcall = call {} @mutate(i64* %load9)
  store {} %fcall, {}* %tmp3
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %deref_load = load i64*, i64** %x
  %load10 = load i64, i64* %deref_load
  store i64 %load10, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define {} @mutate(i64* %0) {
basic_block0:
  %retvar = alloca {}
  %ptr = alloca i64*
  store i64* %0, i64** %ptr
  %ptr1 = alloca i64*
  %load = load i64*, i64** %ptr
  store i64* %load, i64** %ptr1
  %deref_load = load i64*, i64** %ptr1
  store i64 9, i64* %deref_load
  store {} undef, {}* %retvar
  %load_ret = load {}, {}* %retvar
  ret {} %load_ret
}

declare noalias i8* @malloc(i32)
