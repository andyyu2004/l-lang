; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca { i64, { i64, { i64, { i64, i64 } } } }
  %tmp1 = alloca { i64, { i64, { i64, i64 } } }
  %tmp2 = alloca { i64, { i64, i64 } }
  %tmp3 = alloca { i64, i64 }
  %tuple_gep = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp3, i32 0, i32 0
  store i64 4, i64* %tuple_gep
  %tuple_gep4 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp3, i32 0, i32 1
  store i64 5, i64* %tuple_gep4
  %tuple_gep5 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2, i32 0, i32 0
  store i64 3, i64* %tuple_gep5
  %load = load { i64, i64 }, { i64, i64 }* %tmp3
  %tuple_gep6 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2, i32 0, i32 1
  store { i64, i64 } %load, { i64, i64 }* %tuple_gep6
  %tuple_gep7 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1, i32 0, i32 0
  store i64 2, i64* %tuple_gep7
  %load8 = load { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2
  %tuple_gep9 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1, i32 0, i32 1
  store { i64, { i64, i64 } } %load8, { i64, { i64, i64 } }* %tuple_gep9
  %tuple_gep10 = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 0
  store i64 1, i64* %tuple_gep10
  %load11 = load { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1
  %tuple_gep12 = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 1
  store { i64, { i64, { i64, i64 } } } %load11, { i64, { i64, { i64, i64 } } }* %tuple_gep12
  %struct_gep = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 1
  %struct_gep13 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %struct_gep, i32 0, i32 1
  %struct_gep14 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %struct_gep13, i32 0, i32 1
  %struct_gep15 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %struct_gep14, i32 0, i32 1
  %load16 = load i64, i64* %struct_gep15
  store i64 %load16, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
