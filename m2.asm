BITS 16
ORG 0x7C00

start:
    cli

    xor ax, ax
    mov ss, ax
    mov sp, 0x7C00

    ; DS = BIOS load segment
    xor ax, ax
    mov ds, ax

    ; ES = test segment
    mov ax, 0x0800
    mov es, ax

    mov si, message
    xor di, di
    mov cx, message_end - message
    rep movsb

    ; Read copied data
    mov ax, 0x0800
    mov ds, ax
    xor si, si

print:
    lodsb
    test al, al
    jz done

    mov ah, 0x0E
    int 0x10
    jmp print

done:
    hlt

message db "Segment test OK",0
message_end:

times 510-($-$$) db 0
dw 0xAA55