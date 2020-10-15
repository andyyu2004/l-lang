; ModuleID = 'main'
source_filename = "main"

%opaque = type { i64, { i64 } }
%opaque.0 = type { i64, { i64 } }

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

define %opaque @"Either::Left<Option<int>,int>"(%opaque.0 %0) {
basic_blockbb0:
  %retvar = alloca %opaque
  %1 = alloca %opaque.0
  store %opaque.0 %0, %opaque.0* %1
  %discr_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque, %opaque* %retvar, i32 0, i32 1
  %enum_ptr_cast = bitcast { i64 }* %enum_gep to { %opaque.0 }*
  %load = load %opaque.0, %opaque.0* %1
  %enum_content_gep = getelementptr inbounds { %opaque.0 }, { %opaque.0 }* %enum_ptr_cast, i32 0, i32 0
  store %opaque.0 %load, %opaque.0* %enum_content_gep
  %load_ret = load %opaque, %opaque* %retvar
  ret %opaque %load_ret
}

define i64 @"main<>"() {
basic_blockbb0:
  %retvar = alloca i64
  %tmp = alloca %opaque
  %tmp1 = alloca %opaque.0
  %e = alloca %opaque
  %tmp2 = alloca i1
  %tmp3 = alloca i64
  %tmp4 = alloca i1
  %opt = alloca %opaque.0
  %tmp5 = alloca i1
  %tmp6 = alloca i64
  %tmp7 = alloca i1
  %i = alloca i64
  %tmp8 = alloca i1
  %tmp9 = alloca i64
  %tmp10 = alloca i1
  %tmp11 = alloca i1
  %tmp12 = alloca i64
  %tmp13 = alloca i1
  %x = alloca i64
  %fcall = call %opaque.0 @"Option::Some<int>"(i64 88)
  store %opaque.0 %fcall, %opaque.0* %tmp1
  br label %basic_blockbb1

basic_blockbb1:                                   ; preds = %basic_blockbb0
  %load = load %opaque.0, %opaque.0* %tmp1
  %fcall14 = call %opaque @"Either::Left<Option<int>,int>"(%opaque.0 %load)
  store %opaque %fcall14, %opaque* %tmp
  br label %basic_blockbb2

basic_blockbb2:                                   ; preds = %basic_blockbb1
  %load15 = load %opaque, %opaque* %tmp
  store %opaque %load15, %opaque* %e
  br label %basic_blockbb3

basic_blockbb3:                                   ; preds = %basic_blockbb2
  store i1 true, i1* %tmp2
  %discr_gep = getelementptr inbounds %opaque, %opaque* %e, i32 0, i32 0
  %load_discr = load i64, i64* %discr_gep
  store i64 %load_discr, i64* %tmp3
  %load16 = load i64, i64* %tmp3
  %icmp_eq = icmp eq i64 0, %load16
  store i1 %icmp_eq, i1* %tmp4
  %load17 = load i1, i1* %tmp4
  %load18 = load i1, i1* %tmp2
  %and = and i1 %load17, %load18
  store i1 %and, i1* %tmp2
  %struct_gep = getelementptr inbounds %opaque, %opaque* %e, i32 0, i32 1
  %lvalue_pointer_cast = bitcast { i64 }* %struct_gep to { %opaque.0 }*
  %struct_gep19 = getelementptr inbounds { %opaque.0 }, { %opaque.0 }* %lvalue_pointer_cast, i32 0, i32 0
  %load20 = load %opaque.0, %opaque.0* %struct_gep19
  store %opaque.0 %load20, %opaque.0* %opt
  %load21 = load i1, i1* %tmp2
  br i1 %load21, label %basic_blockbb4, label %basic_blockbb5

basic_blockbb4:                                   ; preds = %basic_blockbb3
  br label %basic_blockbb8

basic_blockbb5:                                   ; preds = %basic_blockbb3
  store i1 true, i1* %tmp11
  %discr_gep22 = getelementptr inbounds %opaque, %opaque* %e, i32 0, i32 0
  %load_discr23 = load i64, i64* %discr_gep22
  store i64 %load_discr23, i64* %tmp12
  %load24 = load i64, i64* %tmp12
  %icmp_eq25 = icmp eq i64 1, %load24
  store i1 %icmp_eq25, i1* %tmp13
  %load26 = load i1, i1* %tmp13
  %load27 = load i1, i1* %tmp11
  %and28 = and i1 %load26, %load27
  store i1 %and28, i1* %tmp11
  %struct_gep29 = getelementptr inbounds %opaque, %opaque* %e, i32 0, i32 1
  %struct_gep30 = getelementptr inbounds { i64 }, { i64 }* %struct_gep29, i32 0, i32 0
  %load31 = load i64, i64* %struct_gep30
  store i64 %load31, i64* %x
  %load32 = load i1, i1* %tmp11
  br i1 %load32, label %basic_blockbb6, label %basic_blockbb14

basic_blockbb6:                                   ; preds = %basic_blockbb5
  %load33 = load i64, i64* %x
  store i64 %load33, i64* %retvar
  br label %basic_blockbb7

basic_blockbb7:                                   ; preds = %basic_blockbb12, %basic_blockbb6
  %load_ret = load i64, i64* %retvar
  ret i64 %load_ret

basic_blockbb8:                                   ; preds = %basic_blockbb4
  store i1 true, i1* %tmp5
  %discr_gep34 = getelementptr inbounds %opaque.0, %opaque.0* %opt, i32 0, i32 0
  %load_discr35 = load i64, i64* %discr_gep34
  store i64 %load_discr35, i64* %tmp6
  %load36 = load i64, i64* %tmp6
  %icmp_eq37 = icmp eq i64 0, %load36
  store i1 %icmp_eq37, i1* %tmp7
  %load38 = load i1, i1* %tmp7
  %load39 = load i1, i1* %tmp5
  %and40 = and i1 %load38, %load39
  store i1 %and40, i1* %tmp5
  %struct_gep41 = getelementptr inbounds %opaque.0, %opaque.0* %opt, i32 0, i32 1
  %struct_gep42 = getelementptr inbounds { i64 }, { i64 }* %struct_gep41, i32 0, i32 0
  %load43 = load i64, i64* %struct_gep42
  store i64 %load43, i64* %i
  %load44 = load i1, i1* %tmp5
  br i1 %load44, label %basic_blockbb9, label %basic_blockbb10

basic_blockbb9:                                   ; preds = %basic_blockbb8
  %load45 = load i64, i64* %i
  store i64 %load45, i64* %retvar
  br label %basic_blockbb12

basic_blockbb10:                                  ; preds = %basic_blockbb8
  store i1 true, i1* %tmp8
  %discr_gep46 = getelementptr inbounds %opaque.0, %opaque.0* %opt, i32 0, i32 0
  %load_discr47 = load i64, i64* %discr_gep46
  store i64 %load_discr47, i64* %tmp9
  %load48 = load i64, i64* %tmp9
  %icmp_eq49 = icmp eq i64 1, %load48
  store i1 %icmp_eq49, i1* %tmp10
  %load50 = load i1, i1* %tmp10
  %load51 = load i1, i1* %tmp8
  %and52 = and i1 %load50, %load51
  store i1 %and52, i1* %tmp8
  %load53 = load i1, i1* %tmp8
  br i1 %load53, label %basic_blockbb11, label %basic_blockbb13

basic_blockbb11:                                  ; preds = %basic_blockbb10
  store i64 4, i64* %retvar
  br label %basic_blockbb12

basic_blockbb12:                                  ; preds = %basic_blockbb11, %basic_blockbb9
  br label %basic_blockbb7

basic_blockbb13:                                  ; preds = %basic_blockbb10
  call void @exit(i32 1)
  unreachable

basic_blockbb14:                                  ; preds = %basic_blockbb5
  call void @exit(i32 1)
  unreachable
}

define %opaque.0 @"Option::Some<int>"(i64 %0) {
basic_blockbb0:
  %retvar = alloca %opaque.0
  %1 = alloca i64
  store i64 %0, i64* %1
  %discr_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 0
  store i64 0, i64* %discr_gep
  %enum_gep = getelementptr inbounds %opaque.0, %opaque.0* %retvar, i32 0, i32 1
  %load = load i64, i64* %1
  %enum_content_gep = getelementptr inbounds { i64 }, { i64 }* %enum_gep, i32 0, i32 0
  store i64 %load, i64* %enum_content_gep
  %load_ret = load %opaque.0, %opaque.0* %retvar
  ret %opaque.0 %load_ret
}
