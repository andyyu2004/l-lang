	.text
	.file	"main"
	.globl	rc_release                      # -- Begin function rc_release
	.p2align	4, 0x90
	.type	rc_release,@function
rc_release:                             # @rc_release
	.cfi_startproc
# %bb.0:                                # %rc_release
	lock		subl	$1, (%rsi)
	ja	.LBB0_1
# %bb.2:                                # %free
	jmp	free                            # TAILCALL
.LBB0_1:                                # %ret
	retq
.Lfunc_end0:
	.size	rc_release, .Lfunc_end0-rc_release
	.cfi_endproc
                                        # -- End function
	.globl	print                           # -- Begin function print
	.p2align	4, 0x90
	.type	print,@function
print:                                  # @print
	.cfi_startproc
# %bb.0:                                # %printint
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rdi, %rsi
	movl	$680997, 4(%rsp)                # imm = 0xA6425
	leaq	4(%rsp), %rdi
	xorl	%eax, %eax
	callq	printf
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	print, .Lfunc_end1-print
	.cfi_endproc
                                        # -- End function
	.globl	print_addr                      # -- Begin function print_addr
	.p2align	4, 0x90
	.type	print_addr,@function
print_addr:                             # @print_addr
	.cfi_startproc
# %bb.0:                                # %printint
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rdi, %rsi
	movl	$684069, 4(%rsp)                # imm = 0xA7025
	leaq	4(%rsp), %rdi
	xorl	%eax, %eax
	callq	printf
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end2:
	.size	print_addr, .Lfunc_end2-print_addr
	.cfi_endproc
                                        # -- End function
	.globl	"new<>"                         # -- Begin function new<>
	.p2align	4, 0x90
	.type	"new<>",@function
"new<>":                                # @"new<>"
	.cfi_startproc
# %bb.0:                                # %basic_blockbb0
	movq	%rsi, %rdx
	movl	%edi, %eax
	movl	%eax, %ecx
	andb	$1, %cl
	movb	%cl, -25(%rsp)
	movq	%rsi, -24(%rsp)
	movb	%cl, -16(%rsp)
	movq	%rsi, -8(%rsp)
                                        # kill: def $al killed $al killed $eax
	retq
.Lfunc_end3:
	.size	"new<>", .Lfunc_end3-"new<>"
	.cfi_endproc
                                        # -- End function
	.globl	"Option::Some<int>"             # -- Begin function Option::Some<int>
	.p2align	4, 0x90
	.type	"Option::Some<int>",@function
"Option::Some<int>":                    # @"Option::Some<int>"
	.cfi_startproc
# %bb.0:                                # %basic_blockbb0
	movq	%rdi, -24(%rsp)
	movw	$0, -16(%rsp)
	movq	%rdi, -8(%rsp)
	xorl	%eax, %eax
	movq	%rdi, %rdx
	retq
.Lfunc_end4:
	.size	"Option::Some<int>", .Lfunc_end4-"Option::Some<int>"
	.cfi_endproc
                                        # -- End function
	.globl	_start                          # -- Begin function _start
	.p2align	4, 0x90
	.type	_start,@function
_start:                                 # @_start
	.cfi_startproc
# %bb.0:                                # %basic_blockbb0
	subq	$56, %rsp
	.cfi_def_cfa_offset 64
	callq	"run<>"
	movl	$3, %edi
	callq	"Option::Some<int>"
	movw	%ax, 24(%rsp)
	movq	%rdx, 32(%rsp)
	movw	%ax, 6(%rsp)
	testw	%ax, %ax
	sete	2(%rsp)
	sete	3(%rsp)
	movq	%rdx, 8(%rsp)
	jne	.LBB5_2
# %bb.1:                                # %basic_blockbb3
	movq	8(%rsp), %rax
	movq	%rax, 16(%rsp)
	jmp	.LBB5_4
.LBB5_2:                                # %basic_blockbb4
	movzwl	24(%rsp), %eax
	movw	%ax, 4(%rsp)
	cmpw	$1, %ax
	sete	(%rsp)
	sete	1(%rsp)
	jne	.LBB5_5
# %bb.3:                                # %basic_blockbb5
	movq	$5, 16(%rsp)
.LBB5_4:                                # %basic_blockbb7
	movq	$0, 40(%rsp)
	xorl	%eax, %eax
	addq	$56, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB5_5:                                # %basic_blockbb8
	.cfi_def_cfa_offset 64
	movl	$1, %edi
	callq	exit
.Lfunc_end5:
	.size	_start, .Lfunc_end5-_start
	.cfi_endproc
                                        # -- End function
	.globl	"run<>"                         # -- Begin function run<>
	.p2align	4, 0x90
	.type	"run<>",@function
"run<>":                                # @"run<>"
	.cfi_startproc
# %bb.0:                                # %basic_blockbb0
	subq	$88, %rsp
	.cfi_def_cfa_offset 96
	movq	$4, 48(%rsp)
	movb	$0, 56(%rsp)
	movq	$4, 16(%rsp)
	movb	$0, 24(%rsp)
	movl	$8, %esi
	xorl	%edi, %edi
	callq	"new<>"
	movq	%rdx, 40(%rsp)
	andb	$1, %al
	movb	%al, 32(%rsp)
	movq	%rdx, 8(%rsp)
	movb	%al, (%rsp)
	movq	16(%rsp), %rdi
	callq	print
	movq	8(%rsp), %rdi
	callq	print
	addq	$88, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end6:
	.size	"run<>", .Lfunc_end6-"run<>"
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
