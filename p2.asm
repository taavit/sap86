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
    db 'W'
    db 'e'
    db 'l'
    db 'c'
    db 'o'
    db 'm'
    db 'e'
    db 0
