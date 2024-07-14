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

const TokenType = enum {
    // Single-character tokens. 
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE,
    RIGHT_BRACE, COMMA, DOT,
    MINUS, PLUS, SEMICOLON,
    SLASH, STAR, 
    // One or two character tokens. 
    //"!"
    BANG,
    // "! ="
    BANG_EQUAL, EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL, LESS,
    LESS_EQUAL, 
    // Literals. 
    IDENTIFIER, STRING,
    NUMBER, 
    // Keywords. 
    AND, CLASS,
    ELSE, FALSE, FUN,
    FOR, IF, NIL,
    OR, 
    PRINT, RETURN,
    SUPER, THIS, TRUE, VAR, WHILE, 
    EOF 
};

pub const Token = struct { 
    type: TokenType, 
    lexema: []const u8, 
    line: usize,
    // literal: Object

    pub fn new(vtype: TokenType , vlexema: []const u8, vline: usize) Token{
        return.{
           .type = vtype,  
           .lexema = vlexema,
           .line = vline,
        };
    }
};

