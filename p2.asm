org 0x7C00

lea bx, [data]
mov ah,0eh
loop:
    mov al,[bx]
    test al,al
    jz done
    int 10h
    inc bx
    jmp loop
done:
    hlt
data:
    db "Hello world!",0

times 510-($-$$) db 0
dw 0xAA55