; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i8, { i64 } }
  %none = alloca { i8, { i64 } }
  %tmp1 = alloca { i8, { i64 } }
  %some = alloca { i8, { i64 } }
  %tmp2 = alloca { i8, { i64 } }
  %some2 = alloca { i8, { i64 } }
  %discr_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp, i32 0, i32 0
  store i8 1, i8* %discr_gep
  %enum_gep = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp, i32 0, i32 1
  %enum_ptr_cast = bitcast { i64 }* %enum_gep to {}*
  %load = load { i8, { i64 } }, { i8, { i64 } }* %tmp
  store { i8, { i64 } } %load, { i8, { i64 } }* %none
  %discr_gep3 = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp1, i32 0, i32 0
  store i8 0, i8* %discr_gep3
  %enum_gep4 = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp1, i32 0, i32 1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep4, i32 0, i32 0
  store i64 9, i64* %enum_content_gep
  %load5 = load { i8, { i64 } }, { i8, { i64 } }* %tmp1
  store { i8, { i64 } } %load5, { i8, { i64 } }* %some
  %discr_gep6 = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp2, i32 0, i32 0
  store i8 0, i8* %discr_gep6
  %enum_gep7 = getelementptr inbounds { i8, { i64 } }, { i8, { i64 } }* %tmp2, i32 0, i32 1
  %enum_content_gep8 = getelementptr inbounds { i64 }, { i64 }* %enum_gep7, i32 0, i32 0
  store i64 5, i64* %enum_content_gep8
  %load9 = load { i8, { i64 } }, { i8, { i64 } }* %tmp2
  store { i8, { i64 } } %load9, { i8, { i64 } }* %some2
  store i64 9, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
