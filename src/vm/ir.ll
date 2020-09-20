; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
basic_block0:
  %retvar = alloca i64
  %tmp = alloca i64
  %x = alloca i64
  %tmp1 = alloca <{ i64, i64 }>
  store i64 5, i64* %tmp
  %load = load i64, i64* %tmp
  store i64 %load, i64* %x
  %struct_gep = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 0
  store i64 1, i64* %struct_gep
  %struct_gep2 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 1
  store i64 2, i64* %struct_gep2
  %struct_gep3 = getelementptr inbounds <{ i64, i64 }>, <{ i64, i64 }>* %tmp1, i32 0, i32 1
  %load4 = load i64, i64* %struct_gep3
  store i64 %load4, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_block1:                                     ; No predecessors!
  %load_ret5 = load i64, i64* %retvar
  ret i64 %load_ret5
}
