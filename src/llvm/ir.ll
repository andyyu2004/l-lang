; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64, { %opaque.0* } }
%opaque.0 = type { i64, %opaque }

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

declare i32 @printf()

declare void @abort()

declare void @exit(i32)

define %opaque @"NodeOption::Some"(%opaque.0* %0) {
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

define i64 @main() {
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
  %tmp9 = alloca i1
  %n = alloca %opaque.0*
  %tmp10 = alloca i1
  %tmp11 = alloca i64
  %tmp12 = alloca i1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %tmp1, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %tmp1, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.0* }* %enum_gep to {}*
  %struct_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp, i32 0, i32 0
  store i64 9, i64* %struct_gep
  %load = load %opaque, %opaque* %tmp1
  %struct_gep13 = getelementptr inbounds %opaque.0, %opaque.0* %tmp, i32 0, i32 1
  store %opaque %load, %opaque* %struct_gep13
  %load14 = load %opaque.0, %opaque.0* %tmp
  store %opaque.0 %load14, %opaque.0* %node
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %opaque.0, i32 }* getelementptr ({ %opaque.0, i32 }, { %opaque.0, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %opaque.0, i32 }*
  %rc_gep = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %box, i32 0, i32 0
  store %opaque.0* %box_gep, %opaque.0** %tmp5
  call void @"rc_retain_Node<>"(%opaque.0** %tmp5)
  %load_deref = load %opaque.0*, %opaque.0** %tmp5
  %load15 = load %opaque.0, %opaque.0* %node
  store %opaque.0 %load15, %opaque.0* %load_deref
  %load16 = load %opaque.0*, %opaque.0** %tmp5
  store %opaque.0* %load16, %opaque.0** %tmp4
  call void @"rc_retain_Node<>"(%opaque.0** %tmp4)
  %load17 = load %opaque.0*, %opaque.0** %tmp4
  %fcall = call %opaque @"NodeOption::Some"(%opaque.0* %load17)
  store %opaque %fcall, %opaque* %tmp3
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %struct_gep18 = getelementptr inbounds %opaque.0, %opaque.0* %tmp2, i32 0, i32 0
  store i64 4, i64* %struct_gep18
  %load19 = load %opaque, %opaque* %tmp3
  %struct_gep20 = getelementptr inbounds %opaque.0, %opaque.0* %tmp2, i32 0, i32 1
  store %opaque %load19, %opaque* %struct_gep20
  %load21 = load %opaque.0, %opaque.0* %tmp2
  store %opaque.0 %load21, %opaque.0* %head
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp6
  %struct_gep22 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %discr_gep23 = getelementptr inbounds %opaque, %opaque* %struct_gep22, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep23
  store i64 %load_discr, i64* %tmp7
  %load24 = load i64, i64* %tmp7
  %icmp_eq = icmp eq i64 1, %load24
  store i1 %icmp_eq, i1* %tmp8
  %load25 = load i1, i1* %tmp8
  %load26 = load i1, i1* %tmp6
  %and = and i1 %load25, %load26
  store i1 %and, i1* %tmp6
  store i1 true, i1* %tmp9
  %struct_gep27 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %struct_gep28 = getelementptr inbounds %opaque, %opaque* %struct_gep27, i32 0, i32 1
  %struct_gep29 = getelementptr inbounds { %opaque.0* }, { %opaque.0* }* %struct_gep28, i32 0, i32 0
  %load30 = load %opaque.0*, %opaque.0** %struct_gep29
  store %opaque.0* %load30, %opaque.0** %n
  call void @"rc_retain_Node<>"(%opaque.0** %n)
  %load31 = load i1, i1* %tmp6
  br i1 %load31, label %basic_blockbb3, label %basic_blockbb4

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load_deref32 = load %opaque.0*, %opaque.0** %n
  %struct_gep33 = getelementptr inbounds %opaque.0, %opaque.0* %load_deref32, i32 0, i32 0
  %load34 = load i64, i64* %struct_gep33
  store i64 %load34, i64* %retvar
  br label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp10
  %struct_gep35 = getelementptr inbounds %opaque.0, %opaque.0* %head, i32 0, i32 1
  %discr_gep36 = getelementptr inbounds %opaque, %opaque* %struct_gep35, i32 0, i32 0
  %load_discr37 = load i64, i64* %discr_gep36
  store i64 %load_discr37, i64* %tmp11
  %load38 = load i64, i64* %tmp11
  %icmp_eq39 = icmp eq i64 0, %load38
  store i1 %icmp_eq39, i1* %tmp12
  %load40 = load i1, i1* %tmp12
  %load41 = load i1, i1* %tmp10
  %and42 = and i1 %load40, %load41
  store i1 %and42, i1* %tmp10
  %load43 = load i1, i1* %tmp10
  br i1 %load43, label %basic_blockbb5, label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb4
  store i64 0, i64* %retvar
  br label %basic_blockbb6

basic_blockbb6:                                   ; preds = %basic_blockbb5, %basic_blockbb3
  call void @"rc_release_Node<>"(%opaque.0** %n)
  call void @"rc_release_Node<>"(%opaque.0** %tmp4)
  call void @"rc_release_Node<>"(%opaque.0** %tmp5)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb7:                                   ; preds = %basic_blockbb4
  call void @exit(i32 1)
  unreachable
}

declare noalias i8* @malloc(i32)

define void @"rc_retain_Node<>"(%opaque.0** %0) {
rc_retain_start:
  %load_box = load %opaque.0*, %opaque.0** %0
  %rc_retain_box_cast = bitcast %opaque.0* %load_box to { %opaque.0, i32 }*
  %rc = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  ret void
}

define void @"rc_release_Node<>"(%opaque.0** %0) {
rc_release_start:
  %load_box = load %opaque.0*, %opaque.0** %0
  %rc_release_box_cast = bitcast %opaque.0* %load_box to { %opaque.0, i32 }*
  %rc = getelementptr inbounds { %opaque.0, i32 }, { %opaque.0, i32 }* %rc_release_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %decrement = sub i32 %load_rc, 1
  store i32 %decrement, i32* %rc
  %rc_cmp = icmp eq i32 %decrement, 0
  br i1 %rc_cmp, label %rc_release_free, label %rc_release_ret

rc_release_free:                                  ; preds = %rc_release_start
  ret void

rc_release_ret:                                   ; preds = %rc_release_start
  ret void
}
