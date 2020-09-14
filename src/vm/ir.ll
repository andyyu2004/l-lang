; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca i64
  %tmp1 = alloca { i1, i64 }
  %i = alloca i64
  %tuple_gep = getelementptr inbounds { i1, i64 }, { i1, i64 }* %tmp1, i32 0, i32 0
  store i1 false, i1* %tuple_gep
  %tuple_gep2 = getelementptr inbounds { i1, i64 }, { i1, i64 }* %tmp1, i32 0, i32 1
  store i64 185, i64* %tuple_gep2
  %load = load { i1, i64 }, { i1, i64 }* %tmp1
  %fcall = call i64 @snd({ i1, i64 } %load)
  store i64 %fcall, i64* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load3 = load i64, i64* %tmp
  store i64 %load3, i64* %i
  %load4 = load i64, i64* %i
  store i64 %load4, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define i64 @snd({ i1, i64 } %0) {
basic_block0:
  %retvar = alloca i64
  %"(b, i)" = alloca { i1, i64 }
  store { i1, i64 } %0, { i1, i64 }* %"(b, i)"
  %b = alloca i1
  %i = alloca i64
  %struct_gep = getelementptr inbounds { i1, i64 }, { i1, i64 }* %"(b, i)", i32 0, i32 0
  %load = load i1, i1* %struct_gep
  store i1 %load, i1* %b
  %struct_gep1 = getelementptr inbounds { i1, i64 }, { i1, i64 }* %"(b, i)", i32 0, i32 1
  %load2 = load i64, i64* %struct_gep1
  store i64 %load2, i64* %i
  %load3 = load i64, i64* %i
  store i64 %load3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
