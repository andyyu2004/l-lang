; ModuleID = 'main'
source_filename = "main"

define double @main() {
basic_block0:
  %retvar = alloca double
  %fcall = call double @fib(double 2.000000e+00)
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
  %load = load double, double* %n
  %fcmp_lt = fcmp olt double %load, 1.000000e+00
  switch i1 %fcmp_lt, label %basic_block2 [
    i1 true, label %basic_block1
  ]

basic_block1:                                     ; preds = %basic_block0
  store double 1.000000e+00, double* %retvar
  br label %basic_block3

basic_block2:                                     ; preds = %basic_block0
  store double 2.000000e+00, double* %retvar
  br label %basic_block3

basic_block3:                                     ; preds = %basic_block2, %basic_block1
  %load_ret = load double, double* %retvar
  ret double %load_ret
}
