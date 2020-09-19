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

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i8, { i64 } }
  %opt = alloca { i8, { i64 } }
  %tmp1 = alloca { i64, i64 }
  %x = alloca i64
  %x2 = alloca i64
  %tmp3 = alloca i64 (i64)*
  %f = alloca i64 (i64)*
  %tmp4 = alloca i64 (i64)*
  %g = alloca i64 (i64)*
  %tmp5 = alloca i64
  %fcall = call { i8, { i64 } } @Some(i64 5)
  store { i8, { i64 } } %fcall, { i8, { i64 } }* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load = load { i8, { i64 } }, { i8, { i64 } }* %tmp
  store { i8, { i64 } } %load, { i8, { i64 } }* %opt
  %tuple_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp1, i32 0, i32 0
  store i64 1, i64* %tuple_gep
  %tuple_gep6 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp1, i32 0, i32 1
  store i64 2, i64* %tuple_gep6
  %struct_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp1, i32 0, i32 0
  %load7 = load i64, i64* %struct_gep
  store i64 %load7, i64* %x
  %struct_gep8 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp1, i32 0, i32 1
  %load9 = load i64, i64* %struct_gep8
  store i64 %load9, i64* %x2
  store i64 (i64)* @"<closure>", i64 (i64)** %tmp3
  %load10 = load i64 (i64)*, i64 (i64)** %tmp3
  store i64 (i64)* %load10, i64 (i64)** %f
  store i64 (i64)* @"<closure>", i64 (i64)** %tmp4
  %load11 = load i64 (i64)*, i64 (i64)** %tmp4
  store i64 (i64)* %load11, i64 (i64)** %g
  %load12 = load i64 (i64)*, i64 (i64)** %g
  %fcall13 = call i64 %load12(i64 9)
  store i64 %fcall13, i64* %tmp5
  br label %basic_block2

basic_block2:                                     ; preds = %basic_block1
  %load14 = load i64 (i64)*, i64 (i64)** %f
  %load15 = load i64, i64* %tmp5
  %fcall16 = call i64 %load14(i64 %load15)
  store i64 %fcall16, i64* %retvar
  br label %basic_block3

basic_block3:                                     ; preds = %basic_block2
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block4:                                     ; No predecessors!
  %load_ret17 = load i64, i64* %retvar
  ret i64 %load_ret17
}

define i64 @"<closure>"(i64 %0) {
basic_block0:
  %retvar = alloca i64
  %k = alloca i64
  store i64 %0, i64* %k
  %k1 = alloca i64
  %load = load i64, i64* %k
  store i64 %load, i64* %k1
  %load2 = load i64, i64* %k1
  store i64 %load2, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block03:                                    ; No predecessors!
  %retvar4 = alloca i64
  %k5 = alloca i64
  store i64 %0, i64* %k5
  %k6 = alloca i64
  %load7 = load i64, i64* %k5
  store i64 %load7, i64* %k6
  %load8 = load i64, i64* %k6
  %tmpidd = add i64 1, %load8
  store i64 %tmpidd, i64* %retvar4
  %load_ret9 = load i64, i64* %retvar4
  ret i64 %load_ret9
}

declare i64 @"<closure>.1"(i64)
