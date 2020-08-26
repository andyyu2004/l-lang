; ModuleID = 'main'
source_filename = "main"

define double @main() {
start:
  %retvar = alloca double
  store double 1.000000e+01, double* %retvar
  %load_ret = load double, double* %retvar
  ret double %load_ret
}
