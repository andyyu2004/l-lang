; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64, { i64 } }
%opaque.0 = type { i64, { %opaque.1* } }
%opaque.1 = type { i64, %opaque.0 }
%opaque.2 = type { i64, { i64 } }
%opaque.3 = type { i64, { %opaque.3*, %opaque.3* } }

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

define %opaque @"Option::Some"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque, %opaque* %retvar
  ret %opaque %load_ret
}

define %opaque.0 @"NodeOption::Some"(%opaque.1* %0) {
basic_blockbb0:
  %retvar = alloca %opaque.0
  %1 = alloca %opaque.1*
  store %opaque.1* %0, %opaque.1** %1
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 1
  %load = load %opaque.1*, %opaque.1** %1
  %enum_content_gep = getelementptr inbounds { %opaque.1* }, { %opaque.1* }* %enum_gep, i32 0, i32 0
  store %opaque.1* %load, %opaque.1** %enum_content_gep
  %load_ret = load %opaque.0, %opaque.0* %retvar
  ret %opaque.0 %load_ret
}

define %opaque.2 @"Either::Left"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque.2
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.2, %opaque.2* %retvar
  ret %opaque.2 %load_ret
}

define %opaque.2 @"Either::Right"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque.2
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.2, %opaque.2* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.2, %opaque.2* %retvar
  ret %opaque.2 %load_ret
}

define %opaque.3 @"Expr::Int"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque.3
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.3*, %opaque.3* }* %enum_gep to { i64 }*
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_ptr_cast, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.3, %opaque.3* %retvar
  ret %opaque.3 %load_ret
}

define %opaque.3 @"Expr::Add"(%opaque.3* %0, %opaque.3* %1) {
basic_blockbb0:
  %retvar = alloca %opaque.3
  %2 = alloca %opaque.3*
  store %opaque.3* %0, %opaque.3** %2
  %3 = alloca %opaque.3*
  store %opaque.3* %1, %opaque.3** %3
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 0
  store i64 1, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.3, %opaque.3* %retvar, i32 0, i32 1
  %load = load %opaque.3*, %opaque.3** %2
  %enum_content_gep = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %enum_gep, i32 0, i32 0
  store %opaque.3* %load, %opaque.3** %enum_content_gep
  %load1 = load %opaque.3*, %opaque.3** %3
  %enum_content_gep2 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %enum_gep, i32 0, i32 1
  store %opaque.3* %load1, %opaque.3** %enum_content_gep2
  %load_ret = load %opaque.3, %opaque.3* %retvar
  ret %opaque.3 %load_ret
}

define i64 @main() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca { i64, i64, i64 }
  %tmp1 = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %b = alloca i64
  %tmp4 = alloca i64
  %tmp5 = alloca i1
  %tmp6 = alloca i1
  %a = alloca i64
  %b7 = alloca i64
  %c = alloca i64
  %tmp8 = alloca %opaque.1*
  %tmp9 = alloca %opaque.1*
  %tmp10 = alloca %opaque.0
  %next = alloca %opaque.1*
  %tmp11 = alloca %opaque.1*
  %tmp12 = alloca %opaque.1*
  %tmp13 = alloca %opaque.0
  %node = alloca %opaque.1*
  %tmp14 = alloca i64
  %struct_gep = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 0
  store i64 1, i64* %struct_gep
  %struct_gep15 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 1
  store i64 2, i64* %struct_gep15
  %struct_gep16 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 2
  store i64 3, i64* %struct_gep16
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp1
  store i64 1, i64* %tmp2
  %load = load i64, i64* %tmp2
  %struct_gep17 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 0
  %load18 = load i64, i64* %struct_gep17
  %icmp_eq = icmp eq i64 %load, %load18
  store i1 %icmp_eq, i1* %tmp3
  %load19 = load i1, i1* %tmp3
  %load20 = load i1, i1* %tmp1
  %and = and i1 %load19, %load20
  store i1 %and, i1* %tmp1
  %struct_gep21 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 1
  %load22 = load i64, i64* %struct_gep21
  store i64 %load22, i64* %b
  store i64 3, i64* %tmp4
  %load23 = load i64, i64* %tmp4
  %struct_gep24 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 2
  %load25 = load i64, i64* %struct_gep24
  %icmp_eq26 = icmp eq i64 %load23, %load25
  store i1 %icmp_eq26, i1* %tmp5
  %load27 = load i1, i1* %tmp5
  %load28 = load i1, i1* %tmp1
  %and29 = and i1 %load27, %load28
  store i1 %and29, i1* %tmp1
  %load30 = load i1, i1* %tmp1
  br i1 %load30, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load31 = load i64, i64* %b
  store i64 %load31, i64* %retvar
  br label %basic_blockbb5

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp6
  %struct_gep32 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 0
  %load33 = load i64, i64* %struct_gep32
  store i64 %load33, i64* %a
  %struct_gep34 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 1
  %load35 = load i64, i64* %struct_gep34
  store i64 %load35, i64* %b7
  %struct_gep36 = getelementptr inbounds { i64, i64, i64 }, { i64, i64, i64 }* %tmp, i32 0, i32 2
  %load37 = load i64, i64* %struct_gep36
  store i64 %load37, i64* %c
  %load38 = load i1, i1* %tmp6
  br i1 %load38, label %basic_blockbb4, label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load39 = load i64, i64* %c
  store i64 %load39, i64* %retvar
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb4, %basic_blockbb2
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable

basic_blockbb7:                                   ; No predecessors!
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ %opaque.1, i32 }* getelementptr ({ %opaque.1, i32 }, { %opaque.1, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { %opaque.1, i32 }*
  %rc_gep = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box, i32 0, i32 0
  store %opaque.1* %box_gep, %opaque.1** %tmp9
  call void @"rc_retain_Node<>"(%opaque.1** %tmp9)
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp10, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp10, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.1* }* %enum_gep to {}*
  %load_deref = load %opaque.1*, %opaque.1** %tmp9
  %struct_gep40 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref, i32 0, i32 0
  store i64 22, i64* %struct_gep40
  %load41 = load %opaque.0, %opaque.0* %tmp10
  %struct_gep42 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref, i32 0, i32 1
  store %opaque.0 %load41, %opaque.0* %struct_gep42
  %load43 = load %opaque.1*, %opaque.1** %tmp9
  store %opaque.1* %load43, %opaque.1** %tmp8
  call void @"rc_retain_Node<>"(%opaque.1** %tmp8)
  %load44 = load %opaque.1*, %opaque.1** %tmp8
  store %opaque.1* %load44, %opaque.1** %next
  call void @"rc_retain_Node<>"(%opaque.1** %next)
  %malloccall45 = tail call i8* @malloc(i32 ptrtoint ({ %opaque.1, i32 }* getelementptr ({ %opaque.1, i32 }, { %opaque.1, i32 }* null, i32 1) to i32))
  %box46 = bitcast i8* %malloccall45 to { %opaque.1, i32 }*
  %rc_gep47 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box46, i32 0, i32 1
  store i32 0, i32* %rc_gep47
  %box_gep48 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box46, i32 0, i32 0
  store %opaque.1* %box_gep48, %opaque.1** %tmp12
  call void @"rc_retain_Node<>"(%opaque.1** %tmp12)
  %load49 = load %opaque.1*, %opaque.1** %next
  %fcall = call %opaque.0 @"NodeOption::Some"(%opaque.1* %load49)
  store %opaque.0 %fcall, %opaque.0* %tmp13
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  %load_deref50 = load %opaque.1*, %opaque.1** %tmp12
  %struct_gep51 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref50, i32 0, i32 0
  store i64 6, i64* %struct_gep51
  %load52 = load %opaque.0, %opaque.0* %tmp13
  %struct_gep53 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref50, i32 0, i32 1
  store %opaque.0 %load52, %opaque.0* %struct_gep53
  %load54 = load %opaque.1*, %opaque.1** %tmp12
  store %opaque.1* %load54, %opaque.1** %tmp11
  call void @"rc_retain_Node<>"(%opaque.1** %tmp11)
  %load55 = load %opaque.1*, %opaque.1** %tmp11
  store %opaque.1* %load55, %opaque.1** %node
  call void @"rc_retain_Node<>"(%opaque.1** %node)
  %load56 = load %opaque.1*, %opaque.1** %node
  %fcall57 = call i64 @last(%opaque.1* %load56)
  store i64 %fcall57, i64* %tmp14
  br label %basic_blockbb9

basic_blockbb9:                                   ; preds = %basic_blockbb8
  store i64 8, i64* %retvar
  call void @"rc_release_Node<>"(%opaque.1** %node)
  call void @"rc_release_Node<>"(%opaque.1** %tmp11)
  call void @"rc_release_Node<>"(%opaque.1** %tmp12)
  call void @"rc_release_Node<>"(%opaque.1** %next)
  call void @"rc_release_Node<>"(%opaque.1** %tmp8)
  call void @"rc_release_Node<>"(%opaque.1** %tmp9)
  %load_ret58 = load i64, i64* %retvar
  ret i64 %load_ret58
}

define i64 @last(%opaque.1* %0) {
basic_blockbb0:
  %retvar = alloca i64
  %node = alloca %opaque.1*
  store %opaque.1* %0, %opaque.1** %node
  %node1 = alloca %opaque.1*
  %tmp = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %next = alloca %opaque.1*
  %tmp4 = alloca i1
  %tmp5 = alloca i64
  %tmp6 = alloca i1
  %load = load %opaque.1*, %opaque.1** %node
  store %opaque.1* %load, %opaque.1** %node1
  call void @"rc_retain_Node<>"(%opaque.1** %node1)
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %opaque.1*, %opaque.1** %node1
  %struct_gep = getelementptr inbounds %opaque.1, %opaque.1* %load_deref, i32 0, i32 1
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %struct_gep, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp2
  %load7 = load i64, i64* %tmp2
  %icmp_eq = icmp eq i64 1, %load7
  store i1 %icmp_eq, i1* %tmp3
  %load8 = load i1, i1* %tmp3
  %load9 = load i1, i1* %tmp
  %and = and i1 %load8, %load9
  store i1 %and, i1* %tmp
  %load_deref10 = load %opaque.1*, %opaque.1** %node1
  %struct_gep11 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref10, i32 0, i32 1
  %struct_gep12 = getelementptr inbounds %opaque.0, %opaque.0* %struct_gep11, i32 0, i32 1
  %struct_gep13 = getelementptr inbounds { %opaque.1* }, { %opaque.1* }* %struct_gep12, i32 0, i32 0
  %load14 = load %opaque.1*, %opaque.1** %struct_gep13
  store %opaque.1* %load14, %opaque.1** %next
  call void @"rc_retain_Node<>"(%opaque.1** %next)
  %load15 = load i1, i1* %tmp
  br i1 %load15, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load %opaque.1*, %opaque.1** %next
  %fcall = call i64 @last(%opaque.1* %load16)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb6

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp4
  %load_deref17 = load %opaque.1*, %opaque.1** %node1
  %struct_gep18 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref17, i32 0, i32 1
  %discr_gep19 = getelementptr inbounds %opaque.0, %opaque.0* %struct_gep18, i32 0, i32 0
  %load_discr20 = load i64, i64* %discr_gep19
  store i64 %load_discr20, i64* %tmp5
  %load21 = load i64, i64* %tmp5
  %icmp_eq22 = icmp eq i64 0, %load21
  store i1 %icmp_eq22, i1* %tmp6
  %load23 = load i1, i1* %tmp6
  %load24 = load i1, i1* %tmp4
  %and25 = and i1 %load23, %load24
  store i1 %and25, i1* %tmp4
  %load26 = load i1, i1* %tmp4
  br i1 %load26, label %basic_blockbb4, label %basic_blockbb7

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load_deref27 = load %opaque.1*, %opaque.1** %node1
  %struct_gep28 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref27, i32 0, i32 0
  %load29 = load i64, i64* %struct_gep28
  store i64 %load29, i64* %retvar
  br label %basic_blockbb5

basic_blockbb5:                                   ; preds = %basic_blockbb6, %basic_blockbb4
  call void @"rc_release_Node<>"(%opaque.1** %next)
  call void @"rc_release_Node<>"(%opaque.1** %node1)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb2
  br label %basic_blockbb5

basic_blockbb7:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable
}

define i64 @eval(%opaque.3* %0) {
basic_blockbb0:
  %retvar = alloca i64
  %expr = alloca %opaque.3*
  store %opaque.3* %0, %opaque.3** %expr
  %expr1 = alloca %opaque.3*
  %tmp = alloca i1
  %tmp2 = alloca i64
  %tmp3 = alloca i1
  %i = alloca i64
  %tmp4 = alloca i1
  %tmp5 = alloca i64
  %tmp6 = alloca i1
  %l = alloca %opaque.3*
  %r = alloca %opaque.3*
  %tmp7 = alloca i64
  %tmp8 = alloca i64
  %load = load %opaque.3*, %opaque.3** %expr
  store %opaque.3* %load, %opaque.3** %expr1
  call void @"rc_retain_Expr<>"(%opaque.3** %expr1)
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  store i1 true, i1* %tmp
  %load_deref = load %opaque.3*, %opaque.3** %expr1
  %discr_gep = getelementptr inbounds %opaque.3, %opaque.3* %load_deref, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp2
  %load9 = load i64, i64* %tmp2
  %icmp_eq = icmp eq i64 0, %load9
  store i1 %icmp_eq, i1* %tmp3
  %load10 = load i1, i1* %tmp3
  %load11 = load i1, i1* %tmp
  %and = and i1 %load10, %load11
  store i1 %and, i1* %tmp
  %load_deref12 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep = getelementptr inbounds %opaque.3, %opaque.3* %load_deref12, i32 0, i32 1
  %lvalue_pointer_cast = bitcast { %opaque.3*, %opaque.3* }* %struct_gep to { i64 }*
  %struct_gep13 = getelementptr inbounds { i64 }, { i64 }* %lvalue_pointer_cast, i32 0, i32 0
  %load14 = load i64, i64* %struct_gep13
  store i64 %load14, i64* %i
  %load15 = load i1, i1* %tmp
  br i1 %load15, label %basic_blockbb2, label %basic_blockbb3

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load16 = load i64, i64* %i
  store i64 %load16, i64* %retvar
  br label %basic_blockbb5

basic_blockbb3:                                   ; preds = %basic_blockbb1
  store i1 true, i1* %tmp4
  %load_deref17 = load %opaque.3*, %opaque.3** %expr1
  %discr_gep18 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref17, i32 0, i32 0
  %load_discr19 = load i64, i64* %discr_gep18
  store i64 %load_discr19, i64* %tmp5
  %load20 = load i64, i64* %tmp5
  %icmp_eq21 = icmp eq i64 1, %load20
  store i1 %icmp_eq21, i1* %tmp6
  %load22 = load i1, i1* %tmp6
  %load23 = load i1, i1* %tmp4
  %and24 = and i1 %load22, %load23
  store i1 %and24, i1* %tmp4
  %load_deref25 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep26 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref25, i32 0, i32 1
  %struct_gep27 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %struct_gep26, i32 0, i32 0
  %load28 = load %opaque.3*, %opaque.3** %struct_gep27
  store %opaque.3* %load28, %opaque.3** %l
  call void @"rc_retain_Expr<>"(%opaque.3** %l)
  %load_deref29 = load %opaque.3*, %opaque.3** %expr1
  %struct_gep30 = getelementptr inbounds %opaque.3, %opaque.3* %load_deref29, i32 0, i32 1
  %struct_gep31 = getelementptr inbounds { %opaque.3*, %opaque.3* }, { %opaque.3*, %opaque.3* }* %struct_gep30, i32 0, i32 1
  %load32 = load %opaque.3*, %opaque.3** %struct_gep31
  store %opaque.3* %load32, %opaque.3** %r
  call void @"rc_retain_Expr<>"(%opaque.3** %r)
  %load33 = load i1, i1* %tmp4
  br i1 %load33, label %basic_blockbb4, label %basic_blockbb6

basic_blockbb4:                                   ; preds = %basic_blockbb3
  %load34 = load %opaque.3*, %opaque.3** %l
  %fcall = call i64 @eval(%opaque.3* %load34)
  store i64 %fcall, i64* %tmp7
  br label %basic_blockbb7

basic_blockbb5:                                   ; preds = %basic_blockbb8, %basic_blockbb2
  call void @"rc_release_Expr<>"(%opaque.3** %r)
  call void @"rc_release_Expr<>"(%opaque.3** %l)
  call void @"rc_release_Expr<>"(%opaque.3** %expr1)
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb6:                                   ; preds = %basic_blockbb3
  call void @exit(i32 1)
  unreachable

basic_blockbb7:                                   ; preds = %basic_blockbb4
  %load35 = load %opaque.3*, %opaque.3** %r
  %fcall36 = call i64 @eval(%opaque.3* %load35)
  store i64 %fcall36, i64* %tmp8
  br label %basic_blockbb8

basic_blockbb8:                                   ; preds = %basic_blockbb7
  %load37 = load i64, i64* %tmp7
  %load38 = load i64, i64* %tmp8
  %iadd = add i64 %load37, %load38
  store i64 %iadd, i64* %retvar
  br label %basic_blockbb5
}

declare noalias i8* @malloc(i32)

define void @"rc_retain_Node<>"(%opaque.1** %0) {
rc_retain_start:
  %load_box = load %opaque.1*, %opaque.1** %0
  %rc_retain_box_cast = bitcast %opaque.1* %load_box to { %opaque.1, i32 }*
  %rc = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  ret void
}

define void @"rc_release_Node<>"(%opaque.1** %0) {
rc_release_start:
  %load_box = load %opaque.1*, %opaque.1** %0
  %rc_release_box_cast = bitcast %opaque.1* %load_box to { %opaque.1, i32 }*
  %rc = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %rc_release_box_cast, i32 0, i32 1
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

define void @"rc_retain_Expr<>"(%opaque.3** %0) {
rc_retain_start:
  %load_box = load %opaque.3*, %opaque.3** %0
  %rc_retain_box_cast = bitcast %opaque.3* %load_box to { %opaque.3, i32 }*
  %rc = getelementptr inbounds { %opaque.3, i32 }, { %opaque.3, i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  ret void
}

define void @"rc_release_Expr<>"(%opaque.3** %0) {
rc_release_start:
  %load_box = load %opaque.3*, %opaque.3** %0
  %rc_release_box_cast = bitcast %opaque.3* %load_box to { %opaque.3, i32 }*
  %rc = getelementptr inbounds { %opaque.3, i32 }, { %opaque.3, i32 }* %rc_release_box_cast, i32 0, i32 1
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
