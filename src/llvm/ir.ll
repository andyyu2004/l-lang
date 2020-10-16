; ModuleID = 'main'
source_filename = "main"

%opaque.0 = type { i64, %opaque }
%opaque = type { i64, { %opaque.0* } }

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

define i64 @"main<>"() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %opaque.0
  %tmp1 = alloca %opaque
  %node = alloca %opaque.0
  %tmp2 = alloca %opaque.0
  %tmp3 = alloca %opaque
  %tmp4 = alloca %opaque.0*
  %tmp5 = alloca %opaque.0*
  %head = alloca %opaque.0
  %tmp6 = alloca i1
  %tmp7 = alloca i64
  %tmp8 = alloca i1
  %n = alloca %opaque.0*
  %tmp9 = alloca i1
  %tmp10 = alloca i64
  %tmp11 = alloca i1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %tmp1, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %tmp1, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.0* }* %enum_gep to {}*
  %struct_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp, i32 0, i32 0
  store i64 9, i64* %struct_gep
  %load = load %opaque, %opaque* %tmp1
  %struct_gep12 = getelementptr inbounds %opaque.0, %opaque.0* %tmp, i32 0, i32 1
  store %opaque %load, %opaque* %struct_gep12
  %load13 = load %opaque.0, %opaque.0* %tmp
  store %opaque.0 %load13, %opaque.0* %node
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %opaque.0, i32 }* getelementptr ({ %opaque.0, i32 }, { %opaque.0, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %opaque.0, i32 }*
  %rc_gep = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %box, i32 0, i32 0
  store %opaque.0* %box_gep, %opaque.0** %tmp5
  %load_deref = load %opaque.0*, %opaque.0** %tmp5
  %load14 = load %opaque.0, %opaque.0* %node
  store %opaque.0 %load14, %opaque.0* %load_deref
  %load15 = load %opaque.0*, %opaque.0** %tmp5
  store %opaque.0* %load15, %opaque.0** %tmp4
  %load16 = load %opaque.0*, %opaque.0** %tmp4
  %fcall = call %opaque @"NodeOption::Some<>"(%opaque.0* %load16)
  store %opaque %fcall, %opaque* %tmp3
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %struct_gep17 = getelementptr inbounds %opaque.0, %opaque.0* %tmp2, i32 0, i32 0
  store i64 4, i64* %struct_gep17
  %load18 = load %opaque, %opaque* %tmp3
  %struct_gep19 = getelementptr inbounds %opaque.0, %opaque.0* %tmp2, i32 0, i32 1
  store %opaque %load18, %opaque* %struct_gep19
  %load20 = load %opaque.0, %opaque.0* %tmp2
  store %opaque.0 %load20, %opaque.0* %head
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp6
  %struct_gep21 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %discr_gep22 = getelementptr inbounds %opaque, %opaque* %struct_gep21, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep22
  store i64 %load_discr, i64* %tmp7
  %load23 = load i64, i64* %tmp7
  %icmp_eq = icmp eq i64 1, %load23
  store i1 %icmp_eq, i1* %tmp8
  %load24 = load i1, i1* %tmp8
  %load25 = load i1, i1* %tmp6
  %and = and i1 %load24, %load25
  store i1 %and, i1* %tmp6
  %struct_gep26 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %struct_gep27 = getelementptr inbounds %opaque, %opaque* %struct_gep26, i32 0, i32 1
  %struct_gep28 = getelementptr inbounds { %opaque.0* }, { %opaque.0* }* %struct_gep27, i32 0, i32 0
  %load29 = load %opaque.0*, %opaque.0** %struct_gep28
  store %opaque.0* %load29, %opaque.0** %n
  %load30 = load i1, i1* %tmp6
  br i1 %load30, label %basic_blockbb3, label %basic_blockbb4

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load_deref31 = load %opaque.0*, %opaque.0** %n
  %struct_gep32 = getelementptr inbounds %opaque.0, %opaque.0* %load_deref31, i32 0, i32 0
  %load33 = load i64, i64* %struct_gep32
  store i64 %load33, i64* %retvar
  br label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp9
  %struct_gep34 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %discr_gep35 = getelementptr inbounds %opaque, %opaque* %struct_gep34, i32 0, i32 0
  %load_discr36 = load i64, i64* %discr_gep35
  store i64 %load_discr36, i64* %tmp10
  %load37 = load i64, i64* %tmp10
  %icmp_eq38 = icmp eq i64 0, %load37
  store i1 %icmp_eq38, i1* %tmp11
  %load39 = load i1, i1* %tmp11
  %load40 = load i1, i1* %tmp9
  %and41 = and i1 %load39, %load40
  store i1 %and41, i1* %tmp9
  %load42 = load i1, i1* %tmp9
  br i1 %load42, label %basic_blockbb5, label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb4
  store i64 0, i64* %retvar
  br label %basic_blockbb6

basic_blockbb6:                                   ; preds = %basic_blockbb5, %basic_blockbb3
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb7:                                   ; preds = %basic_blockbb4
  call void @exit(i32 1)
  unreachable
}

define %opaque @"NodeOption::Some<>"(%opaque.0* %0) {
basic_blockbb0:
  %retvar = alloca %opaque
  %1 = alloca %opaque.0*
  store %opaque.0* %0, %opaque.0** %1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 1
  %load = load %opaque.0*, %opaque.0** %1
  %enum_content_gep = getelementptr inbounds { %opaque.0* }, { %opaque.0* }* %enum_gep, i32 0, i32 0
  store %opaque.0* %load, %opaque.0** %enum_content_gep
  %load_ret = load %opaque, %opaque* %retvar
  ret %opaque %load_ret
}

declare noalias i8* @malloc(i32)
