.syntax unified
.arm

.global start
.section .text.start

start:
    // no need to init the sp, it's already done by the BROM
    blx main

.thumb
