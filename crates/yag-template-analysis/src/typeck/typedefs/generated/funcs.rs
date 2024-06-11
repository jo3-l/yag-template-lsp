use crate::typeck::Func;
pub static FUNCS: phf::Map<&'static str, &'static Func> = ::phf::Map {
    key: 12913932095322966823,
    disps: &[
        (3, 93),
        (0, 37),
        (0, 14),
        (0, 136),
        (0, 11),
        (2, 0),
        (1, 37),
        (0, 134),
        (1, 158),
        (0, 93),
        (0, 28),
        (0, 71),
        (3, 182),
        (1, 152),
        (0, 68),
        (0, 38),
        (15, 52),
        (0, 26),
        (1, 50),
        (59, 9),
        (0, 1),
        (0, 6),
        (0, 72),
        (0, 107),
        (0, 39),
        (0, 2),
        (1, 170),
        (22, 162),
        (4, 84),
        (17, 165),
        (127, 137),
        (0, 1),
        (0, 4),
        (0, 118),
        (0, 99),
        (0, 8),
        (3, 130),
        (0, 11),
        (7, 40),
    ],
    entries: &[
        (
            "humanizeDurationSeconds",
            &Func {
                name: "humanizeDurationSeconds",
                doc: "",
            },
        ),
        (
            "bitwiseAnd",
            &Func {
                name: "bitwiseAnd",
                doc: "",
            },
        ),
        (
            "addResponseReactions",
            &Func {
                name: "addResponseReactions",
                doc: "",
            },
        ),
        ("lower", &Func { name: "lower", doc: "" }),
        (
            "getMessage",
            &Func {
                name: "getMessage",
                doc: "",
            },
        ),
        (
            "createForumPost",
            &Func {
                name: "createForumPost",
                doc: "",
            },
        ),
        ("mathConst", &Func { name: "mathConst", doc: "" }),
        (
            "complexMessageEdit",
            &Func {
                name: "complexMessageEdit",
                doc: "",
            },
        ),
        (
            "humanizeThousands",
            &Func {
                name: "humanizeThousands",
                doc: "",
            },
        ),
        ("hasPrefix", &Func { name: "hasPrefix", doc: "" }),
        ("max", &Func { name: "max", doc: "" }),
        ("dbCount", &Func { name: "dbCount", doc: "" }),
        ("sort", &Func { name: "sort", doc: "" }),
        ("verb", &Func { name: "verb", doc: "" }),
        (
            "bitwiseAndNot",
            &Func {
                name: "bitwiseAndNot",
                doc: "",
            },
        ),
        ("carg", &Func { name: "carg", doc: "" }),
        ("urlescape", &Func { name: "urlescape", doc: "" }),
        (
            "getPinCount",
            &Func {
                name: "getPinCount",
                doc: "",
            },
        ),
        (
            "deleteMessage",
            &Func {
                name: "deleteMessage",
                doc: "",
            },
        ),
        (
            "addRoleName",
            &Func {
                name: "addRoleName",
                doc: "",
            },
        ),
        ("upper", &Func { name: "upper", doc: "" }),
        ("sendDM", &Func { name: "sendDM", doc: "" }),
        ("dbDelByID", &Func { name: "dbDelByID", doc: "" }),
        ("execAdmin", &Func { name: "execAdmin", doc: "" }),
        ("noun", &Func { name: "noun", doc: "" }),
        (
            "humanizeTimeSinceDays",
            &Func {
                name: "humanizeTimeSinceDays",
                doc: "",
            },
        ),
        (
            "sendMessageRetID",
            &Func {
                name: "sendMessageRetID",
                doc: "",
            },
        ),
        (
            "getTargetPermissionsIn",
            &Func {
                name: "getTargetPermissionsIn",
                doc: "",
            },
        ),
        (
            "complexMessage",
            &Func {
                name: "complexMessage",
                doc: "",
            },
        ),
        (
            "currentUserAgeMinutes",
            &Func {
                name: "currentUserAgeMinutes",
                doc: "",
            },
        ),
        (
            "publishMessage",
            &Func {
                name: "publishMessage",
                doc: "",
            },
        ),
        ("log", &Func { name: "log", doc: "" }),
        (
            "reFindAllSubmatches",
            &Func {
                name: "reFindAllSubmatches",
                doc: "",
            },
        ),
        ("toString", &Func { name: "toString", doc: "" }),
        (
            "editMessage",
            &Func {
                name: "editMessage",
                doc: "",
            },
        ),
        (
            "pastNicknames",
            &Func {
                name: "pastNicknames",
                doc: "",
            },
        ),
        (
            "targetHasRoleID",
            &Func {
                name: "targetHasRoleID",
                doc: "",
            },
        ),
        (
            "sendMessageNoEscapeRetID",
            &Func {
                name: "sendMessageNoEscapeRetID",
                doc: "",
            },
        ),
        (
            "ne",
            &Func {
                name: "ne",
                doc: "Returns the boolean truth of `arg1 != arg2`.",
            },
        ),
        (
            "snowflakeToTime",
            &Func {
                name: "snowflakeToTime",
                doc: "",
            },
        ),
        (
            "dbDelMultiple",
            &Func {
                name: "dbDelMultiple",
                doc: "",
            },
        ),
        (
            "reQuoteMeta",
            &Func {
                name: "reQuoteMeta",
                doc: "",
            },
        ),
        ("pow", &Func { name: "pow", doc: "" }),
        (
            "sendTemplateDM",
            &Func {
                name: "sendTemplateDM",
                doc: "",
            },
        ),
        (
            "call",
            &Func {
                name: "call",
                doc: "Returns the result of calling the first argument, which must be a function, with the remaining arguments as parameters. Thus `call .X.Y 1 2` is, in Go notation, `dot.X.Y(1, 2)` where Y is a func-valued field, map entry, or the like.\n\nThe first argument must be the result of an evaluation that yields a value of function type (as distinct from a predefined function such as print). The function must return either one or two result values, the second of which is of type error. If the arguments don't match the function or the returned error value is non-nil, execution stops.",
            },
        ),
        (
            "dbGetPatternReverse",
            &Func {
                name: "dbGetPatternReverse",
                doc: "",
            },
        ),
        (
            "getChannelPins",
            &Func {
                name: "getChannelPins",
                doc: "",
            },
        ),
        (
            "index",
            &Func {
                name: "index",
                doc: "Return the result of indexing its first argument by the following arguments. Thus `index x 1 2 3` is, in Go syntax, `x[1][2][3]`. Each indexed item must be a map, slice, or array.",
            },
        ),
        (
            "le",
            &Func {
                name: "le",
                doc: "Returns the boolean truth of `arg1 <= arg2`.",
            },
        ),
        (
            "bitwiseLeftShift",
            &Func {
                name: "bitwiseLeftShift",
                doc: "",
            },
        ),
        ("str", &Func { name: "str", doc: "" }),
        ("addRoleID", &Func { name: "addRoleID", doc: "" }),
        ("dbDel", &Func { name: "dbDel", doc: "" }),
        ("split", &Func { name: "split", doc: "" }),
        (
            "hasPermissions",
            &Func {
                name: "hasPermissions",
                doc: "",
            },
        ),
        (
            "formatTime",
            &Func {
                name: "formatTime",
                doc: "",
            },
        ),
        (
            "getChannel",
            &Func {
                name: "getChannel",
                doc: "",
            },
        ),
        ("bitwiseOr", &Func { name: "bitwiseOr", doc: "" }),
        (
            "editMessageNoEscape",
            &Func {
                name: "editMessageNoEscape",
                doc: "",
            },
        ),
        ("in", &Func { name: "in", doc: "" }),
        (
            "pastUsernames",
            &Func {
                name: "pastUsernames",
                doc: "",
            },
        ),
        ("println", &Func { name: "println", doc: "" }),
        (
            "bitwiseNot",
            &Func {
                name: "bitwiseNot",
                doc: "",
            },
        ),
        (
            "js",
            &Func {
                name: "js",
                doc: "Returns the escaped JavaScript equivalent of the textual representation of its arguments.",
            },
        ),
        (
            "and",
            &Func {
                name: "and",
                doc: "Returns the boolean AND of its arguments by returning the first empty argument or the last argument. That is, `and x y` behaves as `if x then y else x`.\n\nNote that and does not short-circuit: all the arguments are evaluated.",
            },
        ),
        (
            "structToSdict",
            &Func {
                name: "structToSdict",
                doc: "",
            },
        ),
        ("toInt64", &Func { name: "toInt64", doc: "" }),
        (
            "dbTopEntries",
            &Func {
                name: "dbTopEntries",
                doc: "",
            },
        ),
        (
            "dbBottomEntries",
            &Func {
                name: "dbBottomEntries",
                doc: "",
            },
        ),
        (
            "giveRoleID",
            &Func {
                name: "giveRoleID",
                doc: "",
            },
        ),
        ("parseTime", &Func { name: "parseTime", doc: "" }),
        ("sub", &Func { name: "sub", doc: "" }),
        (
            "ge",
            &Func {
                name: "ge",
                doc: "Returns the boolean truth of `arg1 >= arg2`.",
            },
        ),
        ("cbrt", &Func { name: "cbrt", doc: "" }),
        (
            "humanizeDurationMinutes",
            &Func {
                name: "humanizeDurationMinutes",
                doc: "",
            },
        ),
        (
            "hasRoleName",
            &Func {
                name: "hasRoleName",
                doc: "",
            },
        ),
        ("inFold", &Func { name: "inFold", doc: "" }),
        (
            "mentionHere",
            &Func {
                name: "mentionHere",
                doc: "",
            },
        ),
        (
            "sendTemplate",
            &Func {
                name: "sendTemplate",
                doc: "",
            },
        ),
        (
            "currentUserAgeHuman",
            &Func {
                name: "currentUserAgeHuman",
                doc: "",
            },
        ),
        (
            "onlineCount",
            &Func {
                name: "onlineCount",
                doc: "",
            },
        ),
        (
            "addThreadMember",
            &Func {
                name: "addThreadMember",
                doc: "",
            },
        ),
        ("getRole", &Func { name: "getRole", doc: "" }),
        ("randInt", &Func { name: "randInt", doc: "" }),
        ("min", &Func { name: "min", doc: "" }),
        (
            "publishResponse",
            &Func {
                name: "publishResponse",
                doc: "",
            },
        ),
        (
            "unpinMessage",
            &Func {
                name: "unpinMessage",
                doc: "",
            },
        ),
        (
            "bitwiseXor",
            &Func {
                name: "bitwiseXor",
                doc: "",
            },
        ),
        (
            "mentionRoleName",
            &Func {
                name: "mentionRoleName",
                doc: "",
            },
        ),
        ("hasRoleID", &Func { name: "hasRoleID", doc: "" }),
        (
            "currentTime",
            &Func {
                name: "currentTime",
                doc: "",
            },
        ),
        (
            "removeThreadMember",
            &Func {
                name: "removeThreadMember",
                doc: "",
            },
        ),
        ("fdiv", &Func { name: "fdiv", doc: "" }),
        (
            "deleteForumPost",
            &Func {
                name: "deleteForumPost",
                doc: "",
            },
        ),
        ("toByte", &Func { name: "toByte", doc: "" }),
        ("roundEven", &Func { name: "roundEven", doc: "" }),
        ("toInt", &Func { name: "toInt", doc: "" }),
        ("title", &Func { name: "title", doc: "" }),
        (
            "lt",
            &Func {
                name: "lt",
                doc: "Returns the boolean truth of `arg1 < arg2`.",
            },
        ),
        (
            "execTemplate",
            &Func {
                name: "execTemplate",
                doc: "Executes the associated template with the name and context data provided, If the associated template returns a value, `execTemplate` evaluates to that value and `nil` otherwise.",
            },
        ),
        ("dbSet", &Func { name: "dbSet", doc: "" }),
        (
            "html",
            &Func {
                name: "html",
                doc: "Returns the escaped HTML equivalent of the textual representation of its arguments.",
            },
        ),
        ("roleAbove", &Func { name: "roleAbove", doc: "" }),
        ("setRoles", &Func { name: "setRoles", doc: "" }),
        (
            "getChannelOrThread",
            &Func {
                name: "getChannelOrThread",
                doc: "",
            },
        ),
        ("joinStr", &Func { name: "joinStr", doc: "" }),
        (
            "dbSetExpire",
            &Func {
                name: "dbSetExpire",
                doc: "",
            },
        ),
        (
            "takeRoleID",
            &Func {
                name: "takeRoleID",
                doc: "",
            },
        ),
        ("exec", &Func { name: "exec", doc: "" }),
        ("userArg", &Func { name: "userArg", doc: "" }),
        (
            "humanizeDurationHours",
            &Func {
                name: "humanizeDurationHours",
                doc: "",
            },
        ),
        (
            "loadLocation",
            &Func {
                name: "loadLocation",
                doc: "",
            },
        ),
        (
            "createTicket",
            &Func {
                name: "createTicket",
                doc: "",
            },
        ),
        (
            "roundFloor",
            &Func {
                name: "roundFloor",
                doc: "",
            },
        ),
        ("trimSpace", &Func { name: "trimSpace", doc: "" }),
        ("print", &Func { name: "print", doc: "" }),
        (
            "deleteAllMessageReactions",
            &Func {
                name: "deleteAllMessageReactions",
                doc: "",
            },
        ),
        ("shuffle", &Func { name: "shuffle", doc: "" }),
        ("cembed", &Func { name: "cembed", doc: "" }),
        (
            "giveRoleName",
            &Func {
                name: "giveRoleName",
                doc: "",
            },
        ),
        ("parseArgs", &Func { name: "parseArgs", doc: "" }),
        (
            "sendMessageNoEscape",
            &Func {
                name: "sendMessageNoEscape",
                doc: "",
            },
        ),
        (
            "targetHasPermissions",
            &Func {
                name: "targetHasPermissions",
                doc: "",
            },
        ),
        ("reReplace", &Func { name: "reReplace", doc: "" }),
        ("dbIncr", &Func { name: "dbIncr", doc: "" }),
        (
            "createThread",
            &Func {
                name: "createThread",
                doc: "",
            },
        ),
        ("mult", &Func { name: "mult", doc: "" }),
        ("dbGet", &Func { name: "dbGet", doc: "" }),
        ("dbRank", &Func { name: "dbRank", doc: "" }),
        ("json", &Func { name: "json", doc: "" }),
        (
            "urlquery",
            &Func {
                name: "urlquery",
                doc: "Returns the escaped value of the textual representation of its arguments in a form suitable for embedding in a URL query.",
            },
        ),
        (
            "len",
            &Func {
                name: "len",
                doc: "Returns the integer length of its argument.",
            },
        ),
        (
            "deleteTrigger",
            &Func {
                name: "deleteTrigger",
                doc: "",
            },
        ),
        ("reFind", &Func { name: "reFind", doc: "" }),
        ("sdict", &Func { name: "sdict", doc: "" }),
        ("reSplit", &Func { name: "reSplit", doc: "" }),
        (
            "not",
            &Func {
                name: "not",
                doc: "Returns the boolean negation of its argument.",
            },
        ),
        (
            "eq",
            &Func {
                name: "eq",
                doc: "Return the boolean truth of `arg1 == arg2`.\n\nFor simpler multi-way equality tests, eq accepts two or more arguments and compares the second and subsequent to the first, returning in effect\n\n``` arg1==arg2 || arg1==arg3 || arg1==arg4 ... ```\n\n(Unlike with || in Go, however, eq is a function call and all the arguments will be evaluated.)",
            },
        ),
        ("sleep", &Func { name: "sleep", doc: "" }),
        ("kindOf", &Func { name: "kindOf", doc: "" }),
        ("add", &Func { name: "add", doc: "" }),
        (
            "mentionEveryone",
            &Func {
                name: "mentionEveryone",
                doc: "",
            },
        ),
        (
            "bitwiseRightShift",
            &Func {
                name: "bitwiseRightShift",
                doc: "",
            },
        ),
        (
            "targetHasRoleName",
            &Func {
                name: "targetHasRoleName",
                doc: "",
            },
        ),
        (
            "editNickname",
            &Func {
                name: "editNickname",
                doc: "",
            },
        ),
        (
            "currentUserCreated",
            &Func {
                name: "currentUserCreated",
                doc: "",
            },
        ),
        ("roundCeil", &Func { name: "roundCeil", doc: "" }),
        (
            "dbGetPattern",
            &Func {
                name: "dbGetPattern",
                doc: "",
            },
        ),
        (
            "pinMessage",
            &Func {
                name: "pinMessage",
                doc: "",
            },
        ),
        (
            "removeRoleName",
            &Func {
                name: "removeRoleName",
                doc: "",
            },
        ),
        ("seq", &Func { name: "seq", doc: "" }),
        ("toRune", &Func { name: "toRune", doc: "" }),
        (
            "takeRoleName",
            &Func {
                name: "takeRoleName",
                doc: "",
            },
        ),
        ("dbDelById", &Func { name: "dbDelById", doc: "" }),
        (
            "mentionRoleID",
            &Func {
                name: "mentionRoleID",
                doc: "",
            },
        ),
        (
            "deleteResponse",
            &Func {
                name: "deleteResponse",
                doc: "",
            },
        ),
        ("toFloat", &Func { name: "toFloat", doc: "" }),
        (
            "editChannelTopic",
            &Func {
                name: "editChannelTopic",
                doc: "",
            },
        ),
        ("mod", &Func { name: "mod", doc: "" }),
        (
            "jsonToSdict",
            &Func {
                name: "jsonToSdict",
                doc: "",
            },
        ),
        ("execCC", &Func { name: "execCC", doc: "" }),
        ("dict", &Func { name: "dict", doc: "" }),
        (
            "addMessageReactions",
            &Func {
                name: "addMessageReactions",
                doc: "",
            },
        ),
        (
            "or",
            &Func {
                name: "or",
                doc: "Returns the boolean OR of its arguments by returning the first non-empty argument or the last argument, that is, `or x y` behaves as `if x then x else y`. All the arguments are evaluated.",
            },
        ),
        ("div", &Func { name: "div", doc: "" }),
        (
            "scheduleUniqueCC",
            &Func {
                name: "scheduleUniqueCC",
                doc: "",
            },
        ),
        (
            "sendMessage",
            &Func {
                name: "sendMessage",
                doc: "",
            },
        ),
        ("round", &Func { name: "round", doc: "" }),
        (
            "deleteMessageReaction",
            &Func {
                name: "deleteMessageReaction",
                doc: "",
            },
        ),
        ("reFindAll", &Func { name: "reFindAll", doc: "" }),
        (
            "onlineCountBots",
            &Func {
                name: "onlineCountBots",
                doc: "",
            },
        ),
        ("slice", &Func { name: "slice", doc: "" }),
        (
            "removeRoleID",
            &Func {
                name: "removeRoleID",
                doc: "",
            },
        ),
        (
            "urlunescape",
            &Func {
                name: "urlunescape",
                doc: "",
            },
        ),
        (
            "editChannelName",
            &Func {
                name: "editChannelName",
                doc: "",
            },
        ),
        (
            "weekNumber",
            &Func {
                name: "weekNumber",
                doc: "",
            },
        ),
        ("sqrt", &Func { name: "sqrt", doc: "" }),
        (
            "timestampToTime",
            &Func {
                name: "timestampToTime",
                doc: "",
            },
        ),
        (
            "addReactions",
            &Func {
                name: "addReactions",
                doc: "",
            },
        ),
        ("getMember", &Func { name: "getMember", doc: "" }),
        ("printf", &Func { name: "printf", doc: "" }),
        ("newDate", &Func { name: "newDate", doc: "" }),
        ("cslice", &Func { name: "cslice", doc: "" }),
        (
            "deleteThread",
            &Func {
                name: "deleteThread",
                doc: "",
            },
        ),
        (
            "gt",
            &Func {
                name: "gt",
                doc: "Returns the boolean truth of `arg1 > arg2`.",
            },
        ),
        (
            "cancelScheduledUniqueCC",
            &Func {
                name: "cancelScheduledUniqueCC",
                doc: "",
            },
        ),
        (
            "sanitizeText",
            &Func {
                name: "sanitizeText",
                doc: "",
            },
        ),
        ("adjective", &Func { name: "adjective", doc: "" }),
        ("getThread", &Func { name: "getThread", doc: "" }),
        (
            "toDuration",
            &Func {
                name: "toDuration",
                doc: "",
            },
        ),
        ("hasSuffix", &Func { name: "hasSuffix", doc: "" }),
    ],
};
