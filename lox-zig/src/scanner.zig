// zig fmt: off
const std = @import("std");

pub const Scanner = struct {
    source: []const u8,
    spacer: std.mem.SplitIterator(u8, .scalar),

    pub fn new(source: []const u8) Scanner {
        return .{ 
            .source = source, 
            .spacer = std.mem.splitScalar(u8, source, ' ') 
        };
    }

    pub fn next(self: *Scanner) ?[]const u8 {
        return self.spacer.next();
    }
};
