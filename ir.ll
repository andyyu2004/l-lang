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
  %tmp = alloca i64*
  %tmp1 = alloca i64*
  %tmp2 = alloca %opaque.1*
  %tmp3 = alloca %opaque.1*
  %tmp4 = alloca %opaque.0
  %next = alloca %opaque.1*
  %tmp5 = alloca %opaque.1*
  %tmp6 = alloca %opaque.1*
  %tmp7 = alloca %opaque.0
  %node = alloca %opaque.1*
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ i64, i32 }* getelementptr ({ i64, i32 }, { i64, i32 }* null, i32 1) to i32))
  %box = bitcast i8* %malloccall to { i64, i32 }*
  %rc_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 1
  store i32 0, i32* %rc_gep
  %box_gep = getelementptr inbounds { i64, i32 }, { i64, i32 }* %box, i32 0, i32 0
  store i64* %box_gep, i64** %tmp1
  call void @rc_retain_int(i64** %tmp1)
  %load_deref = load i64*, i64** %tmp1
  store i64 5, i64* %load_deref
  %load = load i64*, i64** %tmp1
  store i64* %load, i64** %tmp
  call void @rc_retain_int(i64** %tmp)
  %load8 = load i64*, i64** %tmp
  %fcall = call i64 @rc(i64* %load8)
  store i64 %fcall, i64* %retvar
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb2:                                   ; No predecessors!
  %malloccall9 = tail call i8* @malloc(i32 ptrtoint ({ %opaque.1, i32 }* getelementptr ({ %opaque.1, i32 }, { %opaque.1, i32 }* null, i32 1) to i32))
  %box10 = bitcast i8* %malloccall9 to { %opaque.1, i32 }*
  %rc_gep11 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box10, i32 0, i32 1
  store i32 0, i32* %rc_gep11
  %box_gep12 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box10, i32 0, i32 0
  store %opaque.1* %box_gep12, %opaque.1** %tmp3
  call void @"rc_retain_Node<>"(%opaque.1** %tmp3)
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp4, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.0, %opaque.0* %tmp4, i32 0, i32 1
  %enum_ptr_cast = bitcast { %opaque.1* }* %enum_gep to {}*
  %load_deref13 = load %opaque.1*, %opaque.1** %tmp3
  %struct_gep = getelementptr inbounds %opaque.1, %opaque.1* %load_deref13, i32 0, i32 0
  store i64 22, i64* %struct_gep
  %load14 = load %opaque.0, %opaque.0* %tmp4
  %struct_gep15 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref13, i32 0, i32 1
  store %opaque.0 %load14, %opaque.0* %struct_gep15
  %load16 = load %opaque.1*, %opaque.1** %tmp3
  store %opaque.1* %load16, %opaque.1** %tmp2
  call void @"rc_retain_Node<>"(%opaque.1** %tmp2)
  %load17 = load %opaque.1*, %opaque.1** %tmp2
  store %opaque.1* %load17, %opaque.1** %next
  call void @"rc_retain_Node<>"(%opaque.1** %next)
  %malloccall18 = tail call i8* @malloc(i32 ptrtoint ({ %opaque.1, i32 }* getelementptr ({ %opaque.1, i32 }, { %opaque.1, i32 }* null, i32 1) to i32))
  %box19 = bitcast i8* %malloccall18 to { %opaque.1, i32 }*
  %rc_gep20 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box19, i32 0, i32 1
  store i32 0, i32* %rc_gep20
  %box_gep21 = getelementptr inbounds { %opaque.1, i32 }, { %opaque.1, i32 }* %box19, i32 0, i32 0
  store %opaque.1* %box_gep21, %opaque.1** %tmp6
  call void @"rc_retain_Node<>"(%opaque.1** %tmp6)
  %load22 = load %opaque.1*, %opaque.1** %next
  %fcall23 = call %opaque.0 @"NodeOption::Some"(%opaque.1* %load22)
  store %opaque.0 %fcall23, %opaque.0* %tmp7
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  %load_deref24 = load %opaque.1*, %opaque.1** %tmp6
  %struct_gep25 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref24, i32 0, i32 0
  store i64 6, i64* %struct_gep25
  %load26 = load %opaque.0, %opaque.0* %tmp7
  %struct_gep27 = getelementptr inbounds %opaque.1, %opaque.1* %load_deref24, i32 0, i32 1
  store %opaque.0 %load26, %opaque.0* %struct_gep27
  %load28 = load %opaque.1*, %opaque.1** %tmp6
  store %opaque.1* %load28, %opaque.1** %tmp5
  call void @"rc_retain_Node<>"(%opaque.1** %tmp5)
  %load29 = load %opaque.1*, %opaque.1** %tmp5
  store %opaque.1* %load29, %opaque.1** %node
  call void @"rc_retain_Node<>"(%opaque.1** %node)
  store i64 8, i64* %retvar
  call void @"rc_release_Node<>"(%opaque.1** %node)
  call void @"rc_release_Node<>"(%opaque.1** %tmp5)
  call void @"rc_release_Node<>"(%opaque.1** %tmp6)
  call void @"rc_release_Node<>"(%opaque.1** %next)
  call void @"rc_release_Node<>"(%opaque.1** %tmp2)
  call void @"rc_release_Node<>"(%opaque.1** %tmp3)
  call void @rc_release_int(i64** %tmp)
  call void @rc_release_int(i64** %tmp1)
  %load_ret30 = load i64, i64* %retvar
  ret i64 %load_ret30
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

define void @rc_retain_int(i64** %0) {
rc_retain_start:
  %load_box = load i64*, i64** %0
  %rc_retain_box_cast = bitcast i64* %load_box to { i64, i32 }*
  %rc = getelementptr inbounds { i64, i32 }, { i64, i32 }* %rc_retain_box_cast, i32 0, i32 1
  %load_rc = load i32, i32* %rc
  %increment_rc = add i32 %load_rc, 1
  store i32 %increment_rc, i32* %rc
  ret void
}

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

define void @rc_release_int(i64** %0) {
rc_release_start:
  %load_box = load i64*, i64** %0
  %rc_release_box_cast = bitcast i64* %load_box to { i64, i32 }*
  %rc = getelementptr inbounds { i64, i32 }, { i64, i32 }* %rc_release_box_cast, i32 0, i32 1
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
