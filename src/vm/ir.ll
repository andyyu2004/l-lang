; ModuleID = 'main'
source_filename = "main"

define double @"0"() {
body:
  %fcall = call double @"0.6"()
  %tmpadd = fadd double %fcall, 2.000000e+00
  ret double %tmpadd
}

define double @"0.6"() {
body:
  ret double 5.000000e+00
}
