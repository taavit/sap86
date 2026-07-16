lea data
mov bx,ax
loop:
    mov al,[bx]
    test ax,ax
    jz done
    int 10h
    inc bx
    jmp loop
done:
    hlt
data:
    db "Hello world!",0

