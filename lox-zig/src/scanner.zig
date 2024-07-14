// zig fmt: off
const std = @import("std");

pub const Cursor = struct {
    source: []const u8,
    position: usize,
    prev: ?u8,
    curr: ?u8,

    pub fn peek(self: *const Cursor) ?u8 {
        return self.peek_nth(0);
    }

    pub fn peek_nth(self: *const Cursor, nth: usize) ?u8 {
        if (self.position + nth < self.source.len) {
            return self.source[self.position + nth];
        }

        return null;
    }

    pub fn bumb(self: *Cursor) void {
        if (self.position < self.source.len) {
            self.prev = self.curr;
            self.curr = self.source[self.position];
            self.position += 1;
        }
    }

    pub fn next(self: *Cursor) ?u8 {
        if (self.position < self.source.len) {
            self.prev = self.curr;
            self.curr = self.source[self.position];
            self.position += 1;

            return self.curr;
        }

        return null;
    }

    pub fn new(source: []const u8) Cursor {
        return .{
          .source = source,
          .position = 0,
          .prev = null,
          .curr= null
        };
    }
};

pub const Scanner = struct {
    cursor: Cursor,

    pub fn new(source: []const u8) Scanner {
        return .{
            .cursor = Cursor.new(source)
        };
    }

    pub fn parse_space(self: *Scanner) TokenType {
        const EMPTY: []const u8  = " \t\r\n";
        while (self.cursor.peek()) |c_peek| {
            _ = std.mem.indexOfScalar(u8, EMPTY, c_peek) orelse break;

            self.cursor.bumb();
        }

        return TokenType.Whitespace;
    }

    pub fn next(self: *Scanner) ?Result {
        const c = self.cursor.next() orelse return null;
        const start = self.cursor.position - 1;

        const res = switch (c) {
            ' ', '\t', '\r', '\n' => self.parse_space(),
            '(' => TokenType.LeftParen,
            ')' => TokenType.RightParen,
            '{' => TokenType.LeftBrace,
            '}' => TokenType.RightBrace,
            ',' => TokenType.Comma,
            '.' => TokenType.Dot,
            '-' => TokenType.Minus,
            '+' => TokenType.Plus,
            ';' => TokenType.Semicolon,
            '*' => TokenType.Star,
            '!' => switch (self.cursor.peek() orelse 0) {
                '=' => blk: {
                    self.cursor.bumb();
                    break :blk TokenType.BangEqual;
                },
                else => TokenType.Bang,
            },
            '=' => switch (self.cursor.peek() orelse 0) {
                '=' => blk: {
                    self.cursor.bumb();
                    break :blk TokenType.EqualEqual;
                },
                else => TokenType.Equal,
            },
            '<' => switch (self.cursor.peek() orelse 0) {
                '=' => blk: {
                    self.cursor.bumb();
                    break :blk TokenType.LessEqual;
                },
                else => TokenType.Less,
            },
            '>' => switch (self.cursor.peek() orelse 0) {
                '=' => blk: {
                    self.cursor.bumb();
                    break :blk TokenType.GreaterEqual;
                },
                else => TokenType.Greater,
            },
            '/' => switch (self.cursor.peek() orelse 0) {
                '/' => blk: {
                    while (self.cursor.peek() orelse 0 != '\n') {
                        self.cursor.bumb();
                    }
                    break :blk TokenType.CommentLine;
                },
                else => TokenType.Slash,
            },
            '"' => blk: while (true) {
                switch (self.cursor.peek() orelse 
                    return Result{
                        .Err = Error.new
                        (ErrorKind.UnfinishString, .{
                       .start = start,
                       .end = self.cursor.position, 
                    })}
                ) {
                    '"' =>{
                          self.cursor.bumb();
                          break :blk TokenType.String;  
                    },
                    else => self.cursor.bumb(),
                }
            },
            else => {
                return Result{.Err = Error.new
                    (ErrorKind.Unknown, .{
                   .start = start,
                   .end = self.cursor.position, 
                })};
            },
        };

        return Result{.Ok = Token.new(res, .{
           .start = start,
           .end = self.cursor.position, 
        })};
    } 
};

pub const TokenType = enum(u8) {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace,
    RightBrace, Comma, Dot,
    Minus, Plus, Semicolon,
    Slash, Star,
    // One Or Two Character Tokens.
    //"!"
    Bang,
    CommentLine,
    // "! ="
    BangEqual, Equal, EqualEqual,
    Greater, GreaterEqual, Less,
    LessEqual,
    // Literals.
    Identifier, String,
    Number,
    // Keywords.
    And, Class,
    Else, False, Fun,
    For, If, Nil,
    Or,
    Print, Return,
    Super, This, True, Var, While,
    Eof, Whitespace
};

pub const ErrorKind = enum(u8){
    Unknown,
    UnfinishString,
};

pub const Error = struct {
   kind : ErrorKind,
   range : Range, 

   fn new(kind : ErrorKind, range : Range) Error{
       return.{
           .kind = kind,
           .range = range,
       };
   } 
};

pub const Result = union(enum){
    Ok:Token,
    Err:Error,
};

pub const Range = struct {
    start: usize,
    end: usize
};

pub const Token = struct {
    type: TokenType,
    range: Range,

    pub fn new(vtype: TokenType , range: Range) Token{
        return.{
           .type = vtype,
           .range = range
        };
    }
};

