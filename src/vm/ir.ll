; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i64, i1 }*
  %tmp1 = alloca { i64, i1 }
  %s = alloca { i64, i1 }*
  %tmp2 = alloca i64*
  %tmp3 = alloca i64
  %ref = alloca i64*
  %tmp4 = alloca {}
  %struct_gep = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 0
  store i64 5, i64* %struct_gep
  %struct_gep5 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep5
  %load = load { i64, i1 }, { i64, i1 }* %tmp1
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i1 }* getelementptr ({ i64, i1 }, { i64, i1 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i1 }*
  store { i64, i1 } %load, { i64, i1 }* %box
  store { i64, i1 }* %box, { i64, i1 }** %tmp
  %load6 = load { i64, i1 }*, { i64, i1 }** %tmp
  store { i64, i1 }* %load6, { i64, i1 }** %s
  store i64 5, i64* %tmp3
  store i64* %tmp3, i64** %tmp2
  %load7 = load i64*, i64** %tmp2
  store i64* %load7, i64** %ref
  %load8 = load i64*, i64** %ref
  %fcall = call {} @mutate(i64* %load8)
  store {} %fcall, {}* %tmp4
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %deref_load = load i64*, i64** %ref
  %load9 = load i64, i64* %deref_load
  store i64 %load9, i64* %retvar
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
