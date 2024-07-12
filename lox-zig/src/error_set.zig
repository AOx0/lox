const std = @import("std");
const assert = std.debug.assert;

pub fn containsAll(comptime ErrorSet: type, comptime ErrorSetToCheckBeingFullyContained: type) bool {
    comptime assert(@typeInfo(ErrorSet) == .ErrorSet);
    comptime assert(@typeInfo(ErrorSetToCheckBeingFullyContained) == .ErrorSet);
    return (ErrorSet || ErrorSetToCheckBeingFullyContained) == ErrorSet;
}

pub fn contains(comptime ErrorSet: type, error_to_check_being_contained: anyerror) bool {
    const ErrorToCheckBeingContained = @TypeOf(@field(anyerror, error_to_check_being_contained));
    return containsAll(ErrorSet, ErrorToCheckBeingContained);
}
