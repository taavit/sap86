lea data
mov d,a
loop:
    ld d
    test a
    jz done
    int 1
    inc d
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

