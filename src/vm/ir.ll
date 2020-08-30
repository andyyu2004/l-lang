; ModuleID = 'main'
source_filename = "main"

define double @main() {
basic_block0:
  %retvar = alloca double
  %"mut x" = alloca double
  %tmp = alloca double
  %tmp1 = alloca double
  store double 6.900000e+01, double* %"mut x"
  %load = load double, double* %"mut x"
  %tmpadd = fadd double %load, 1.000000e+00
  store double %tmpadd, double* %"mut x"
  %load2 = load double, double* %"mut x"
  %tmpadd3 = fadd double %load2, 5.000000e+00
  store double %tmpadd3, double* %tmp1
  switch i1 false, label %basic_block4 [
    i1 true, label %basic_block1
    i1 false, label %basic_block2
  ]

basic_block1:                                     ; preds = %basic_block0
  store double 5.000000e+00, double* %retvar
  br label %basic_block3

basic_block2:                                     ; preds = %basic_block0
  store double 4.000000e+00, double* %retvar
  br label %basic_block3

basic_block3:                                     ; preds = %basic_block2, %basic_block1
  %load_ret = load double, double* %retvar
  ret double %load_ret

basic_block4:                                     ; preds = %basic_block0
  unreachable
}
