; ModuleID = 'main'
source_filename = "main"

define double @main() {
start:
  %retvar = alloca double
  %"mut x" = alloca double
  %tmp = alloca double
  store double 6.900000e+01, double* %"mut x"
  %load = load double, double* %"mut x"
  %tmpadd = fadd double %load, 1.000000e+00
  store double %tmpadd, double* %"mut x"
  %load1 = load double, double* %"mut x"
  %tmpadd2 = fadd double %load1, 5.000000e+00
  store double %tmpadd2, double* %retvar
  %load_ret = load double, double* %retvar
  ret double %load_ret
}
