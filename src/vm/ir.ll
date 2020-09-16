; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i8, { i64 } }
  %none = alloca { i8, { i64 } }
  %discr_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp, i32 0, i32 0
  store i8 1, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp, i32 0, i32 1
  %enum_ptr_cast = bitcast { i64 }* %enum_gep to {}*
  %load = load { i8, { i64 } }, { i8, { i64 } }* %tmp
  store { i8, { i64 } } %load, { i8, { i64 } }* %none
  store i64 5, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
