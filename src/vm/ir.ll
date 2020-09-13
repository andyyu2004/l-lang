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
  %ld = load { i1, i64 }, { i1, i64 }* %tmp1
  %fcall = call i64 @snd({ i1, i64 } %ld)
  store i64 %fcall, i64* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %ld3 = load i64, i64* %tmp
  store i64 %ld3, i64* %i
  %ld4 = load i64, i64* %i
  store i64 %ld4, i64* %retvar
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
  %ld = load i1, i1* %struct_gep
  store i1 %ld, i1* %b
  %struct_gep1 = getelementptr inbounds { i1, i64 }, { i1, i64 }* %"(b, i)", i32 0, i32 1
  %ld2 = load i64, i64* %struct_gep1
  store i64 %ld2, i64* %i
  %ld3 = load i64, i64* %i
  store i64 %ld3, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
