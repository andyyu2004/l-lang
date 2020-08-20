; ModuleID = 'main'
source_filename = "main"

define double @main() {
body:
  %fcall = call double @fib(double 4.000000e+01)
  ret double %fcall
}

define double @fib(double %"1.5") {
body:
  %fcmp_lt = fcmp olt double %"1.5", 2.000000e+00
  br i1 %fcmp_lt, label %match_end, label %arm_1

arm_1:                                            ; preds = %body
  %tmpfsub = fadd double %"1.5", -1.000000e+00
  %fcall = call double @fib(double %tmpfsub)
  %tmpfsub5 = fadd double %"1.5", -2.000000e+00
  %fcall6 = call double @fib(double %tmpfsub5)
  %tmpadd = fadd double %fcall, %fcall6
  br label %match_end

match_end:                                        ; preds = %body, %arm_1
  %match_phi = phi double [ %tmpadd, %arm_1 ], [ %"1.5", %body ]
  ret double %match_phi
}
