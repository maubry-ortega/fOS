const UART0 = @as(*volatile u8, @ptrFromInt(0xFE20_1000));

export fn _start() callconv(.C) noreturn {
    const msg = "Hello from FossaOS .pe!\n";
    for (msg) |c| {
        UART0.* = c;
    }
    while (true) {}
}