const std = @import("std");
const cio = @import("io.zig");
const Scanner = @import("scanner.zig").Scanner;

const MAX_USIZE = std.math.maxInt(usize);
const Allocator = std.heap.page_allocator;
// const Allocator = std.heap.c_allocator;

var had_error = false;

const AppErrorKind = error{ WrongArgs, CompError, FileRead, OutOfMemory };
const AppError = union(enum) { WrongArgs, CompError: struct { errors: std.ArrayList(CompError) }, FileRead: struct { kind: anyerror, file: []const u8 } };

const CompErrorKind = error{Syntax};
const CompError = union(enum) {
    Syntax: struct {
        path: []const u8,
        line: usize,
        col: usize,
    },

    fn display(self: @This()) void {
        switch (self) {
            .Syntax => {
                cio.println("{s}:{}:{} Error de sintaxis", .{ self.Syntax.path, self.Syntax.line, self.Syntax.col });
            },
        }
    }
};

const RunError = error{CompError};

pub fn report(line: usize, where: []const u8, message: []const u8) !void {
    cio.eprintln("[line: {}] Error {s}: {s}", .{ line, where, message });
    had_error = true;
}

pub fn perr(line: usize, message: []const u8) !void {
    try report(line, "", message);
}

pub fn app(args: [][]const u8, diag: *?AppError) AppErrorKind!void {
    var comp_diag: ?std.ArrayList(CompError) = null;
    switch (args.len) {
        0 => runRepl(),
        1 => runFile(args[0], &comp_diag) catch |err| {
            switch (err) {
                error.FileNotFound, error.AccessDenied, error.FileTooBig, error.IsDir => {
                    diag.* = AppError{ .FileRead = .{ .kind = err, .file = args[0] } };
                    return AppErrorKind.FileRead;
                },
                error.CompError => {
                    diag.* = AppError{ .CompError = .{ .errors = comp_diag.? } };
                    return AppErrorKind.CompError;
                },
                else => unreachable,
            }
        },
        else => {
            diag.* = AppError.WrongArgs;
            return AppErrorKind.WrongArgs;
        },
    }
}

pub fn main() !void {
    cio.init();
    var args_iter = try std.process.argsWithAllocator(Allocator);
    defer args_iter.deinit();

    if (!args_iter.skip()) @panic("Error no se encontro la ruta del ejecutable");

    var args = std.ArrayList([]const u8).init(Allocator);
    defer args.deinit();

    while (args_iter.next()) |arg| try args.append(arg);

    var diagnostics: ?AppError = null;

    app(args.items, &diagnostics) catch |err| {
        const diag = diagnostics.?;

        switch (err) {
            AppErrorKind.WrongArgs => {
                cio.eprintln("Error: Usage lox <FILE>", .{});
            },
            AppErrorKind.CompError => {
                for (diag.CompError.errors.items) |cerr| cerr.display();
                diag.CompError.errors.deinit();
            },
            AppErrorKind.FileRead => {
                cio.eprintln("Error: Failed to open file \"{s}\": {}", .{ diag.FileRead.file, diag.FileRead.kind });
            },
            AppErrorKind.OutOfMemory => {},
        }
    };
}

fn runRepl() void {
    var bin = std.io.bufferedReader(std.io.getStdIn().reader());
    var bw = bin.reader();

    while (true) {
        cio.print("> ", .{});
        cio.flush_out();

        const line = bw.readUntilDelimiterAlloc(Allocator, '\n', MAX_USIZE) catch break;
        defer Allocator.free(line);

        var diagnostics: ?std.ArrayList(CompError) = null;
        if (run("REPL", line, &diagnostics)) {} else |_| {
            const diag = diagnostics.?;
            for (diag.items) |err| err.display();
            diag.deinit();
        }
    }
}

fn run(from: []const u8, source: []const u8, diag: *?std.ArrayList(CompError)) RunError!void {
    var scanner = Scanner.new(source);

    while (scanner.next()) |token| {
        cio.println("{s}", .{token});
    }

    diag.* = std.ArrayList(CompError).init(Allocator);
    diag.*.?.append(CompError{ .Syntax = .{ .path = from, .line = 10, .col = 2 } }) catch @panic("Error");

    return RunError.CompError;
}

const CompOrReadError = error{ FileNotFound, AccessDenied, FileTooBig, IsDir } || RunError;

fn runFile(ruta: []const u8, diag: *?std.ArrayList(CompError)) CompOrReadError!void {
    var cwd = std.fs.cwd();

    const file = cwd.openFile(ruta, .{ .mode = .read_only }) catch |err| switch (err) {
        error.FileNotFound => return error.FileNotFound,
        error.AccessDenied => return error.AccessDenied,
        error.FileTooBig => return error.FileTooBig,
        error.IsDir => return error.IsDir,
        else => @panic("Error opening file"),
    };
    const contents = file.readToEndAlloc(Allocator, MAX_USIZE) catch @panic("Error");
    file.close();

    defer Allocator.free(contents);

    try run(ruta, contents, diag);
}
