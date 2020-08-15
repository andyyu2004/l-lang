; ModuleID = 'main'
source_filename = "main"

define double @main() {
body:
  %fcall = call double ()* @mk_counter()
  %fcall1 = call double %fcall()
  %fcall3 = call double %fcall()
  %fcall5 = call double %fcall()
  %fcall7 = call double %fcall()
  ret double %fcall7
}

define double ()* @mk_counter() {
body:
  ret double ()* @"1.15"
}

define double @"1.15"() {
body:
  %load = load double, double* undef
  %tmpadd = fadd double %load, 1.000000e+00
  store double %tmpadd, double* undef
  ret double %tmpadd
}
