; ModuleID = 'main'
source_filename = "main"

define double @"0"() {
body:
  %fcall = call double ()* @"1"()
  %fcall1 = call double %fcall()
  ret double %fcall1
}

define double ()* @"1"() {
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
