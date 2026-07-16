lea data
mov dx,ax
loop:
    ld dx
    test ax
    jz done
    int 10h
    inc dx
    jmp loop
done:
    hlt
data:
    db "Hello world!",0

