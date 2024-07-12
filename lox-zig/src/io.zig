const std = @import("std");

var bout: std.io.BufferedWriter(4096, std.fs.File.Writer) = undefined;
var berr: std.io.BufferedWriter(4096, std.fs.File.Writer) = undefined;
var stdout: std.io.AnyWriter = undefined;
var stderr: std.io.AnyWriter = undefined;

pub fn init() void {
    bout = std.io.bufferedWriter(std.io.getStdOut().writer());
    berr = std.io.bufferedWriter(std.io.getStdErr().writer());

    stdout = bout.writer().any();
    stderr = berr.writer().any();
}

pub fn print(comptime format: []const u8, args: anytype) void {
    stdout.print(format, args) catch unreachable;
}

pub fn println(comptime format: []const u8, args: anytype) void {
    print(format, args);
    print("\n", .{});
    bout.flush() catch unreachable;
}

pub fn eprintln(comptime format: []const u8, args: anytype) void {
    stderr.print(format, args) catch unreachable;
    stderr.print("\n", .{}) catch unreachable;
    berr.flush() catch unreachable;
}

pub fn flush_out() void {
    bout.flush() catch unreachable;
}
