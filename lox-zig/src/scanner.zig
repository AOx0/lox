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
    start: usize,

    pub fn new(source: []const u8) Scanner {
        return .{
            .cursor = Cursor.new(source),
            .start = 0,
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

    pub fn on_match(s: *@This(), char: u8, tt: TokenType) ?TokenType {
        if (s.cursor.peek() orelse 0 == char) {
            s.cursor.bumb();
            return tt;
        } else {
            return null;
        }
    }

    fn try_parse_comment(s: *@This()) ?TokenType {
        if (s.cursor.peek() orelse 0 == '/') {
            while (s.cursor.peek() orelse '\n' != '\n') s.cursor.bumb();
            return TokenType.CommentLine;
        }

        return null;
    }

    fn try_parse_string(s: *@This()) ?TokenType {
        while (true) {
            s.cursor.bumb();
            if (s.cursor.peek() orelse { return null; } == '"') {
                s.cursor.bumb();
                return TokenType.String;  
            }
        }
    }

    fn bump_while(s: *@This(), f: fn(u8) bool) void {
        while (f(s.cursor.peek() orelse 0)) {
            s.cursor.bumb();
        }
    }

    fn try_parse_number(s: *@This()) ?TokenType {
        var seen_dot = false;
        while (true) switch (s.cursor.peek() orelse 0) {
            '0'...'9' => { s.cursor.bumb(); },
            '.' => switch (s.cursor.peek_nth(1) orelse 0) {
                '0'...'9' => {
                    s.cursor.bumb();
                    if (seen_dot) {
                        s.bump_while(struct { fn f(c: u8) bool {
                            return switch (c) {
                                '0'...'9' => true,
                                '.' => true,
                                else => false
                            };
                        } }.f);
                        return null;
                    }
                    seen_dot = true;
                },
                // Allow `9.9.sqrt()`
                //       `9.sqrt()`
                else => return .Number,
            },
            else => return .Number,
        };
    }

    fn prod_err(s: *@This(), kind: ErrorKind) Result {
        return Result{
            .Err = Error.new(kind, .{
                .start = s.start,
                .end = s.cursor.position
            })
        };
    }

    fn parse_reserved(s: *@This()) ?TokenType {
        s.bump_while(struct { fn f(c: u8) bool {
            return switch (c) {
                '0'...'9', 'a'...'z', 'A'...'Z', '_' => true,
                else => false,
            };
        }}.f);

        const string: []const u8 = s.cursor.source[s.start..s.cursor.position];

        switch (string.len) {
            2 => {
                if (std.mem.eql(u8, string, "if")) return .If;
                if (std.mem.eql(u8, string, "or")) return .Or;
            },
            3 => {
                if (std.mem.eql(u8, string, "and")) return .And;
                if (std.mem.eql(u8, string, "for")) return .For;
                if (std.mem.eql(u8, string, "fun")) return .Fun;
                if (std.mem.eql(u8, string, "var")) return .Var;
                if (std.mem.eql(u8, string, "nil")) return .Nil;
            },
            4 => {
                if (std.mem.eql(u8, string, "else")) return .Else;
                if (std.mem.eql(u8, string, "true")) return .True;
                if (std.mem.eql(u8, string, "this")) return .This;
            },
            5 => {
                if (std.mem.eql(u8, string, "class")) return .Class;
                if (std.mem.eql(u8, string, "false")) return .False;
                if (std.mem.eql(u8, string, "print")) return .Print;
                if (std.mem.eql(u8, string, "super")) return .Super;
                if (std.mem.eql(u8, string, "while")) return .While;
            },
            6 => {
                if (std.mem.eql(u8, string, "return")) return .Return;
            },
            else => return null,
        }
            
        return null;
    }

    pub fn next(s: *Scanner) ?Result {
        const c = s.cursor.next() orelse return null;
        s.start = s.cursor.position - 1;

        const res = switch (c) {
            'a'...'z', 'A'...'Z', '_' => s.parse_reserved() orelse TokenType.Identifier,
            ' ', '\t', '\r', '\n' => s.parse_space(),
            '0'...'9' => s.try_parse_number() orelse return s.prod_err(.InvalidNumber),
            '(' => .LeftParen,
            ')' => .RightParen,
            '{' => .LeftBrace,
            '}' => .RightBrace,
            ',' => .Comma,
            '.' => .Dot,
            '-' => .Minus,
            '+' => .Plus,
            ';' => .Semicolon,
            '*' => .Star,
            '!' => s.on_match('=', .BangEqual) orelse .Bang,
            '=' => s.on_match('=', .EqualEqual) orelse .Equal,
            '<' => s.on_match('=', .LessEqual) orelse .Less,
            '>' => s.on_match('=', .GreaterEqual) orelse .Greater,
            '/' => s.try_parse_comment() orelse .Slash,
            '"' => s.try_parse_string() orelse return s.prod_err(.UnfinishString),
            else => return s.prod_err(.Unknown)
        };

        return Result{.Ok = Token.new(res, .{
           .start = s.start,
           .end = s.cursor.position, 
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
    InvalidNumber,
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

