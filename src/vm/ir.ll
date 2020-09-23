; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca <{ i64, <{ i64, i1, i64 }> }>
  %tmp1 = alloca <{ i64, i1, i64 }>
  %struct_gep = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 0
  store i64 4, i64* %struct_gep
  %struct_gep2 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep2
  %struct_gep3 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 2
  store i64 9, i64* %struct_gep3
  %struct_gep4 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp, i32 0, i32 0
  store i64 5, i64* %struct_gep4
  %load = load <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1
  %struct_gep5 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp, i32 0, i32 1
  store <{ i64, i1, i64 }> %load, <{ i64, i1, i64 }>* %struct_gep5
  %struct_gep6 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp, i32 0, i32 1
  %struct_gep7 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %struct_gep6, i32 0, i32 2
  %load8 = load i64, i64* %struct_gep7
  store i64 %load8, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
