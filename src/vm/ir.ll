; ModuleID = 'main'
source_filename = "main"

define double @main() {
start:
  %retvar = alloca double
  %x = alloca double
  %y = alloca double
  store double 2.000000e+00, double* %x
  store double 4.000000e+00, double* %y
  %load = load double, double* %x
  %load1 = load double, double* %y
  %tmpadd = fadd double %load, %load1
  store double %tmpadd, double* %retvar
  %load_ret = load double, double* %retvar
  ret double %load_ret
}
