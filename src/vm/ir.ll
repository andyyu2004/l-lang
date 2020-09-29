; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca <{ i64, <{ i64, i1, i64 }> }>
  %tmp1 = alloca <{ i64, i1, i64 }>
  %s = alloca <{ i64, <{ i64, i1, i64 }> }>
  %tmp2 = alloca <{ <{ i64, <{ i64, i1, i64 }> }> }>
  %t = alloca <{ <{ i64, <{ i64, i1, i64 }> }> }>
  %struct_gep = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 0
  store i64 1, i64* %struct_gep
  %struct_gep3 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 1
  store i1 false, i1* %struct_gep3
  %struct_gep4 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1, i32 0, i32 2
  store i64 3, i64* %struct_gep4
  %struct_gep5 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp, i32 0, i32 0
  store i64 5, i64* %struct_gep5
  %load = load <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %tmp1
  %struct_gep6 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp, i32 0, i32 1
  store <{ i64, i1, i64 }> %load, <{ i64, i1, i64 }>* %struct_gep6
  %load7 = load <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %tmp
  store <{ i64, <{ i64, i1, i64 }> }> %load7, <{ i64, <{ i64, i1, i64 }> }>* %s
  %load8 = load <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %s
  %struct_gep9 = getelementptr inbounds <{ <{ i64, <{ i64, i1, i64 }> }> }>, <{ <{ i64, <{ i64, i1, i64 }> }> }>* %tmp2, i32 0, i32 0
  store <{ i64, <{ i64, i1, i64 }> }> %load8, <{ i64, <{ i64, i1, i64 }> }>* %struct_gep9
  %load10 = load <{ <{ i64, <{ i64, i1, i64 }> }> }>, <{ <{ i64, <{ i64, i1, i64 }> }> }>* %tmp2
  store <{ <{ i64, <{ i64, i1, i64 }> }> }> %load10, <{ <{ i64, <{ i64, i1, i64 }> }> }>* %t
  %struct_gep11 = getelementptr inbounds <{ <{ i64, <{ i64, i1, i64 }> }> }>, <{ <{ i64, <{ i64, i1, i64 }> }> }>* %t, i32 0, i32 0
  %struct_gep12 = getelementptr inbounds <{ i64, <{ i64, i1, i64 }> }>, <{ i64, <{ i64, i1, i64 }> }>* %struct_gep11, i32 0, i32 1
  %struct_gep13 = getelementptr inbounds <{ i64, i1, i64 }>, <{ i64, i1, i64 }>* %struct_gep12, i32 0, i32 2
  %load14 = load i64, i64* %struct_gep13
  store i64 %load14, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
