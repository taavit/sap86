lea data
mov bx,ax
loop:
    mov ax,[bx]
    test ax,ax
    jz done
    int 10h
    inc bx
    jmp loop
done:
    hlt
data:
    db "Hello world!",0

