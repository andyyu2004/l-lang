	.text
	.file	"main"
	.globl	rc_release              # -- Begin function rc_release
	.p2align	4, 0x90
	.type	rc_release,@function
rc_release:                             # @rc_release
	.cfi_startproc
# %bb.0:                                # %rc_release
	lock		subl	$1, (%rsi)
	ja	.LBB0_1
# %bb.2:                                # %free
	jmp	free                    # TAILCALL
.LBB0_1:                                # %ret
	retq
.Lfunc_end0:
	.size	rc_release, .Lfunc_end0-rc_release
	.cfi_endproc
                                        # -- End function
	.globl	main                    # -- Begin function main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# %bb.0:                                # %basic_block0
	movq	$4, -64(%rsp)
	movb	$0, -56(%rsp)
	movq	$9, -55(%rsp)
	movq	$5, -40(%rsp)
	movq	$4, -32(%rsp)
	movb	$0, -24(%rsp)
	movq	$9, -23(%rsp)
	movq	$9, -8(%rsp)
	movl	$9, %eax
	retq
.Lfunc_end1:
	.size	main, .Lfunc_end1-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
