org 7C00h

start:
    cli

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 7C00h

    sti

    mov si, message

print_loop:
    lodsb               ; AL = [DS:SI], SI++

    cmp al, 0
    jz done

    call putchar
    jmp print_loop

done:
    hlt
    jmp $

putchar:
    push ax
    push bx

    mov ah, 0Eh
    mov bh, 0
    mov bl, 07h
    int 10h

    pop bx
    pop ax
    ret

message:
    db 'Hello from emulator!',13,10,0

times 510-($-$$) db 0
dw 0AA55h