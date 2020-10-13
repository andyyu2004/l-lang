; ModuleID = 'main'
source_filename = "main"

define void @rc_release(i8* %0, i32* %1) {
rc_release:
  %2 = atomicrmw sub i32* %1, i32 1 seq_cst
  %rc_cmp = icmp ule i32 %2, 1
  br i1 %rc_cmp, label %free, label %ret

free:                                             ; preds = %rc_release
  tail call void @free(i8* %0)
  ret void

ret:                                              ; preds = %rc_release
  ret void
}

declare void @free(i8*)

declare i32 @printf(i8*)

declare void @abort()

declare void @exit(i32)

define i64 @rc(i64* %0) {
rc_entry:
  %sdf = bitcast i64* %0 to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %sdf, i32 0, i32 1
  %load_refcount = load i32, i32* %rc_gep
  %"rc->i64" = sext i32 %load_refcount to i64
  ret i64 %"rc->i64"
}

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca { i64, { i64, { i64, { i64, i64 } } } }
  %tmp1 = alloca { i64, { i64, { i64, i64 } } }
  %tmp2 = alloca { i64, { i64, i64 } }
  %tmp3 = alloca { i64, i64 }
  %struct_gep = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 0
  store i64 1, i64* %struct_gep
  %struct_gep4 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1, i32 0, i32 0
  store i64 2, i64* %struct_gep4
  %struct_gep5 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2, i32 0, i32 0
  store i64 3, i64* %struct_gep5
  %struct_gep6 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp3, i32 0, i32 0
  store i64 4, i64* %struct_gep6
  %struct_gep7 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %tmp3, i32 0, i32 1
  store i64 5, i64* %struct_gep7
  %struct_gep8 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2, i32 0, i32 1
  %load = load { i64, i64 }, { i64, i64 }* %tmp3
  store { i64, i64 } %load, { i64, i64 }* %struct_gep8
  %struct_gep9 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1, i32 0, i32 1
  %load10 = load { i64, { i64, i64 } }, { i64, { i64, i64 } }* %tmp2
  store { i64, { i64, i64 } } %load10, { i64, { i64, i64 } }* %struct_gep9
  %struct_gep11 = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 1
  %load12 = load { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %tmp1
  store { i64, { i64, { i64, i64 } } } %load12, { i64, { i64, { i64, i64 } } }* %struct_gep11
  %struct_gep13 = getelementptr inbounds { i64, { i64, { i64, { i64, i64 } } } }, { i64, { i64, { i64, { i64, i64 } } } }* %tmp, i32 0, i32 1
  %struct_gep14 = getelementptr inbounds { i64, { i64, { i64, i64 } } }, { i64, { i64, { i64, i64 } } }* %struct_gep13, i32 0, i32 1
  %struct_gep15 = getelementptr inbounds { i64, { i64, i64 } }, { i64, { i64, i64 } }* %struct_gep14, i32 0, i32 1
  %struct_gep16 = getelementptr inbounds { i64, i64 }, { i64, i64 }* %struct_gep15, i32 0, i32 1
  %load17 = load i64, i64* %struct_gep16
  store i64 %load17, i64* %retvar
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret
}
