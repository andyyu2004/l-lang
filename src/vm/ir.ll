; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { double, { i64, i1 } }
  %f = alloca double
  %i = alloca i64
  %b = alloca i1
  %fcall = call { double, { i64, i1 } } @mk_nested_tuple()
  store { double, { i64, i1 } } %fcall, { double, { i64, i1 } }* %tmp
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %struct_gep = getelementptr inbounds { double, { i64, i1 } }, { double, { i64, i1 } }* %tmp, i32 0, i32 0
  %ld = load double, double* %struct_gep
  store double %ld, double* %f
  %struct_gep1 = getelementptr inbounds { double, { i64, i1 } }, { double, { i64, i1 } }* %tmp, i32 0, i32 1
  %struct_gep2 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %struct_gep1, i32 0, i32 0
  %ld3 = load i64, i64* %struct_gep2
  store i64 %ld3, i64* %i
  %struct_gep4 = getelementptr inbounds { double, { i64, i1 } }, { double, { i64, i1 } }* %tmp, i32 0, i32 1
  %struct_gep5 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %struct_gep4, i32 0, i32 1
  %ld6 = load i1, i1* %struct_gep5
  store i1 %ld6, i1* %b
  %ld7 = load i64, i64* %i
  store i64 %ld7, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}

define { double, { i64, i1 } } @mk_nested_tuple() {
basic_block0:
  %retvar = alloca { double, { i64, i1 } }
  %tmp = alloca { i64, i1 }
  %tuple_gep = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp, i32 0, i32 0
  store i64 30, i64* %tuple_gep
  %tuple_gep1 = getelementptr inbounds { i64, i1 }, { i64, i1 }* %tmp, i32 0, i32 1
  store i1 false, i1* %tuple_gep1
  %tuple_gep2 = getelementptr inbounds { double, { i64, i1 } }, { double, { i64, i1 } }* %retvar, i32 0, i32 0
  store double 9.000000e+01, double* %tuple_gep2
  %ld = load { i64, i1 }, { i64, i1 }* %tmp
  %tuple_gep3 = getelementptr inbounds { double, { i64, i1 } }, { double, { i64, i1 } }* %retvar, i32 0, i32 1
  store { i64, i1 } %ld, { i64, i1 }* %tuple_gep3
  %load_ret = load { double, { i64, i1 } }, { double, { i64, i1 } }* %retvar
  ret { double, { i64, i1 } } %load_ret
}
