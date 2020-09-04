; ModuleID = 'main'
source_filename = "main"

define double @main() {
basic_block0:
  %retvar = alloca double
  %fcall = call double @fib(double 1.000000e+01)
  store double %fcall, double* %retvar
  br label %basic_block1

basic_block1:                                     ; preds = %basic_block0
  %load_ret = load double, double* %retvar
  ret double %load_ret
}

define double @fib(double %0) {
basic_block0:
  %retvar = alloca double
  %n = alloca double
  store double %0, double* %n
  %tmp = alloca double
  %tmp1 = alloca double
  %tmp2 = alloca double
  %tmp3 = alloca double
  %load = load double, double* %n
  %fcmp_lt = fcmp olt double %load, 2.000000e+00
  switch i1 %fcmp_lt, label %basic_block3 [
    i1 true, label %basic_block2
  ]

basic_block1:                                     ; preds = %basic_block5, %basic_block2
  %load_ret = load double, double* %retvar
  ret double %load_ret

basic_block2:                                     ; preds = %basic_block0
  %load4 = load double, double* %n
  store double %load4, double* %retvar
  br label %basic_block1

basic_block3:                                     ; preds = %basic_block0
  %load5 = load double, double* %n
  %tmpfsub = fsub double %load5, 1.000000e+00
  store double %tmpfsub, double* %tmp1
  %load6 = load double, double* %tmp1
  %fcall = call double @fib(double %load6)
  store double %fcall, double* %tmp
  br label %basic_block4

basic_block4:                                     ; preds = %basic_block3
  %load7 = load double, double* %n
  %tmpfsub8 = fsub double %load7, 2.000000e+00
  store double %tmpfsub8, double* %tmp3
  %load9 = load double, double* %tmp3
  %fcall10 = call double @fib(double %load9)
  store double %fcall10, double* %tmp2
  br label %basic_block5

basic_block5:                                     ; preds = %basic_block4
  %load11 = load double, double* %tmp
  %load12 = load double, double* %tmp2
  %tmpadd = fadd double %load11, %load12
  store double %tmpadd, double* %retvar
  br label %basic_block1
}
