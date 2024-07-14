// zig fmt: off

const std = @import("std");
const cio = @import("io.zig");
const scanner = @import("scanner.zig");
const Scanner = scanner.Scanner;
const TokenType = scanner.TokenType;

const MAX_USIZE = std.math.maxInt(usize);
const Allocator = std.heap.page_allocator;
// const Allocator = std.heap.c_allocator;

var had_error = false;

const RunError = error{CompError};

const AppErrorKind = error{
    WrongArgs, CompError,
    FileRead, OutOfMemory
};
const AppError = union(enum) {
    WrongArgs,
    CompError: struct { errors: std.ArrayList(CompError) },
    FileRead: struct { kind: anyerror, file: []const u8 }
}; // AppError

const CompErrorKind = error{Syntax};
const CompError = union(enum) {
    ScannerError: struct {
        path: []const u8,
        line: usize,
        col: usize,
        token: []const u8,
        err: scanner.ErrorKind,
    },

    fn display(self: @This()) void {
        switch (self) {
            .ScannerError => {
                cio.println("{s}:{}:{} Scanner error with token {s}: {any}", .{ 
                    self.ScannerError.path, 
                    self.ScannerError.line, 
                    self.ScannerError.col,
                    self.ScannerError.token,
                    self.ScannerError.err 
                });
            }
        }
    }
}; // CompError


pub fn report(line: usize, where: []const u8, message: []const u8) !void {
    cio.eprintln("[line: {}] Error {s}: {s}", .{ line, where, message });
    had_error = true;
}

pub fn perr(line: usize, message: []const u8) !void {
    try report(line, "", message);
}

pub fn app(args: [][]const u8, buf: *std.ArrayList(u8), diag: *AppError) AppErrorKind!void {
    var comp_diag: std.ArrayList(CompError) = undefined;
    switch (args.len) {
        0 => runRepl(),
        1 => runFile(args[0], buf, &comp_diag) catch |err| {
            switch (err) {
                error.FileNotFound, error.AccessDenied, error.FileTooBig, error.IsDir => {
                    diag.* = AppError{ .FileRead = .{ .kind = err, .file = args[0] } };
                    return AppErrorKind.FileRead;
                },
                error.CompError => {
                    diag.* = AppError{ .CompError = .{ .errors = comp_diag } };
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
} // app

pub fn main() !void {
    cio.init();
    var buf = std.ArrayList(u8).init(Allocator);
    defer buf.deinit();
    
    var args_iter = try std
        .process
        .argsWithAllocator(Allocator);
    defer args_iter.deinit();

    if (!args_iter.skip()) 
        @panic("Error no se encontro la ruta del ejecutable");

    var args = std
        .ArrayList([]const u8)
        .init(Allocator);
    defer args.deinit();

    while (args_iter.next()) |arg| try args.append(arg);

    var diag: AppError = undefined;
    app(args.items, &buf, &diag) catch |err| {
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
    }; // catch err to return from app
} // main

fn runRepl() void {
    var bin = std.io.bufferedReader(std.io.getStdIn().reader());
    var bw = bin.reader();

    while (true) {
        cio.print("> ", .{});
        cio.flush_out();

        const line = bw.readUntilDelimiterAlloc(Allocator, '\n', MAX_USIZE) catch break;
        defer Allocator.free(line);

        var diag: std.ArrayList(CompError) = undefined;
        if (run("REPL", line, &diag)) {} else |_| {
            for (diag.items) |err| err.display();
            diag.deinit();
        }
    } // while input line by the user
} // runRepl()

fn run(from: []const u8, source: []const u8, diag: *std.ArrayList(CompError)) RunError!void {
    var scan = Scanner.new(source);
    diag.* = std.ArrayList(CompError).init(Allocator);
    defer if (diag.items.len == 0) diag.deinit();
    
    while (scan.next()) |result| {
        switch (result) {
            .Ok => {
                cio.println("{any}", .{result.Ok});
            },
            .Err => {
                 diag.append(CompError{.ScannerError = .{
                     .path = from, 
                     .line = 0, 
                     .col = 0, 
                     .token = source[result.Err.range.start .. result.Err.range.end],
                     .err =  result.Err.kind,
                 }}) catch @panic("Failed to push");
                
            }
        }
    }
    if (diag.items.len != 0) return RunError.CompError;
}

const CompOrReadError = error{ FileNotFound, AccessDenied, FileTooBig, IsDir } || RunError;

fn runFile(ruta: []const u8, buf: *std.ArrayList(u8), diag: *std.ArrayList(CompError)) CompOrReadError!void {
    var cwd = std.fs.cwd();

    const file = cwd.openFile(ruta, .{ .mode = .read_only }) catch |err| switch (err) {
        error.FileNotFound => return error.FileNotFound,
        error.AccessDenied => return error.AccessDenied,
        error.FileTooBig => return error.FileTooBig,
        error.IsDir => return error.IsDir,
        else => @panic("Error opening file"),
    };

    var lbuf: [500]u8 = std.mem.zeroes([500]u8);
    while (file.read(&lbuf) catch null) |n| {
        if (n == 0) break;
        
        cio.println("Read {} bytes", .{n});
        buf.appendSlice(lbuf[0..n]) catch @panic("Failed to append to buf");
    }

    file.close();

    try run(ruta, buf.items, diag);
}
