; ModuleID = 'main'
source_filename = "main"

define { i8, { i64 } } @Some(i64 %0) {
basic_block0:
  %retvar = alloca { i8, { i64 } }
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %retvar, i32 0, i32 0
  store i8 0, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load { i8, { i64 } }, { i8, { i64 } }* %retvar
  ret { i8, { i64 } } %load_ret
}

define { i8, { i64 } } @None() {
basic_block0:
  %retvar = alloca { i8, { i64 } }
  %discr_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %retvar, i32 0, i32 0
  store i8 1, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %retvar, i32 0, i32 1
  %enum_ptr_cast = bitcast { i64 }* %enum_gep to {}*
  %load_ret = load { i8, { i64 } }, { i8, { i64 } }* %retvar
  ret { i8, { i64 } } %load_ret
}

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i8, { i64 } } (i64)*
  %opt = alloca { i8, { i64 } } (i64)*
  %tmp1 = alloca { i8, { i64 } }
  store { i8, { i64 } } (i64)* @Some, { i8, { i64 } } (i64)** %tmp
  %load = load { i8, { i64 } } (i64)*, { i8, { i64 } } (i64)** %tmp
  store { i8, { i64 } } (i64)* %load, { i8, { i64 } } (i64)** %opt
  %load2 = load { i8, { i64 } } (i64)*, { i8, { i64 } } (i64)** %opt
  %fcall = call { i8, { i64 } } %load2(i64 5)
  store { i8, { i64 } } %fcall, { i8, { i64 } }* %tmp1
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  store i64 5, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
