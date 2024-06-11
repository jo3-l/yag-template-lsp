use crate::typeck::Func;
pub const DEFINED_FUNCS: &'static [&Func] = &[
    &Func {
        name: "and",
        doc: "Returns the boolean AND of its arguments by returning the first empty argument or the last argument. That is, \"and x y\" behaves as \"if x then y else x.\" Evaluation proceeds through the arguments left to right and returns when the result is determined.",
    },
    &Func { name: "call", doc: "" },
    &Func {
        name: "execTemplate",
        doc: "",
    },
    &Func { name: "html", doc: "" },
    &Func { name: "index", doc: "" },
    &Func { name: "js", doc: "" },
    &Func { name: "len", doc: "" },
    &Func { name: "not", doc: "" },
    &Func { name: "or", doc: "" },
    &Func { name: "urlquery", doc: "" },
    &Func { name: "eq", doc: "" },
    &Func { name: "ge", doc: "" },
    &Func { name: "gt", doc: "" },
    &Func { name: "le", doc: "" },
    &Func { name: "lt", doc: "" },
    &Func { name: "ne", doc: "" },
    &Func {
        name: "editMessage",
        doc: "",
    },
    &Func {
        name: "editMessageNoEscape",
        doc: "",
    },
    &Func {
        name: "pinMessage",
        doc: "",
    },
    &Func {
        name: "publishMessage",
        doc: "",
    },
    &Func {
        name: "publishResponse",
        doc: "",
    },
    &Func { name: "sendDM", doc: "" },
    &Func {
        name: "sendMessage",
        doc: "",
    },
    &Func {
        name: "sendMessageNoEscape",
        doc: "",
    },
    &Func {
        name: "sendMessageNoEscapeRetID",
        doc: "",
    },
    &Func {
        name: "sendMessageRetID",
        doc: "",
    },
    &Func {
        name: "sendTemplate",
        doc: "",
    },
    &Func {
        name: "sendTemplateDM",
        doc: "",
    },
    &Func {
        name: "unpinMessage",
        doc: "",
    },
    &Func {
        name: "mentionEveryone",
        doc: "",
    },
    &Func {
        name: "mentionHere",
        doc: "",
    },
    &Func {
        name: "mentionRoleID",
        doc: "",
    },
    &Func {
        name: "mentionRoleName",
        doc: "",
    },
    &Func { name: "hasRoleID", doc: "" },
    &Func {
        name: "hasRoleName",
        doc: "",
    },
    &Func { name: "addRoleID", doc: "" },
    &Func {
        name: "removeRoleID",
        doc: "",
    },
    &Func { name: "setRoles", doc: "" },
    &Func {
        name: "addRoleName",
        doc: "",
    },
    &Func {
        name: "removeRoleName",
        doc: "",
    },
    &Func {
        name: "giveRoleID",
        doc: "",
    },
    &Func {
        name: "giveRoleName",
        doc: "",
    },
    &Func {
        name: "takeRoleID",
        doc: "",
    },
    &Func {
        name: "takeRoleName",
        doc: "",
    },
    &Func {
        name: "targetHasRoleID",
        doc: "",
    },
    &Func {
        name: "targetHasRoleName",
        doc: "",
    },
    &Func {
        name: "hasPermissions",
        doc: "",
    },
    &Func {
        name: "targetHasPermissions",
        doc: "",
    },
    &Func {
        name: "getTargetPermissionsIn",
        doc: "",
    },
    &Func {
        name: "addMessageReactions",
        doc: "",
    },
    &Func {
        name: "addReactions",
        doc: "",
    },
    &Func {
        name: "addResponseReactions",
        doc: "",
    },
    &Func {
        name: "deleteAllMessageReactions",
        doc: "",
    },
    &Func {
        name: "deleteMessage",
        doc: "",
    },
    &Func {
        name: "deleteMessageReaction",
        doc: "",
    },
    &Func {
        name: "deleteResponse",
        doc: "",
    },
    &Func {
        name: "deleteTrigger",
        doc: "",
    },
    &Func {
        name: "getChannel",
        doc: "",
    },
    &Func {
        name: "getChannelPins",
        doc: "",
    },
    &Func {
        name: "getChannelOrThread",
        doc: "",
    },
    &Func { name: "getMember", doc: "" },
    &Func {
        name: "getMessage",
        doc: "",
    },
    &Func {
        name: "getPinCount",
        doc: "",
    },
    &Func { name: "getRole", doc: "" },
    &Func { name: "getThread", doc: "" },
    &Func {
        name: "createThread",
        doc: "",
    },
    &Func {
        name: "deleteThread",
        doc: "",
    },
    &Func {
        name: "addThreadMember",
        doc: "",
    },
    &Func {
        name: "removeThreadMember",
        doc: "",
    },
    &Func {
        name: "createForumPost",
        doc: "",
    },
    &Func {
        name: "deleteForumPost",
        doc: "",
    },
    &Func {
        name: "currentUserAgeHuman",
        doc: "",
    },
    &Func {
        name: "currentUserAgeMinutes",
        doc: "",
    },
    &Func {
        name: "currentUserCreated",
        doc: "",
    },
    &Func { name: "reFind", doc: "" },
    &Func { name: "reFindAll", doc: "" },
    &Func {
        name: "reFindAllSubmatches",
        doc: "",
    },
    &Func { name: "reReplace", doc: "" },
    &Func { name: "reSplit", doc: "" },
    &Func { name: "sleep", doc: "" },
    &Func {
        name: "editChannelName",
        doc: "",
    },
    &Func {
        name: "editChannelTopic",
        doc: "",
    },
    &Func {
        name: "editNickname",
        doc: "",
    },
    &Func {
        name: "onlineCount",
        doc: "",
    },
    &Func {
        name: "onlineCountBots",
        doc: "",
    },
    &Func { name: "sort", doc: "" },
    &Func {
        name: "pastUsernames",
        doc: "",
    },
    &Func {
        name: "pastNicknames",
        doc: "",
    },
    &Func {
        name: "createTicket",
        doc: "",
    },
    &Func { name: "exec", doc: "" },
    &Func { name: "execAdmin", doc: "" },
    &Func { name: "userArg", doc: "" },
    &Func { name: "parseArgs", doc: "" },
    &Func { name: "carg", doc: "" },
    &Func { name: "execCC", doc: "" },
    &Func {
        name: "scheduleUniqueCC",
        doc: "",
    },
    &Func {
        name: "cancelScheduledUniqueCC",
        doc: "",
    },
    &Func { name: "dbSet", doc: "" },
    &Func {
        name: "dbSetExpire",
        doc: "",
    },
    &Func { name: "dbIncr", doc: "" },
    &Func { name: "dbGet", doc: "" },
    &Func {
        name: "dbGetPattern",
        doc: "",
    },
    &Func {
        name: "dbGetPatternReverse",
        doc: "",
    },
    &Func { name: "dbDel", doc: "" },
    &Func { name: "dbDelById", doc: "" },
    &Func { name: "dbDelByID", doc: "" },
    &Func {
        name: "dbDelMultiple",
        doc: "",
    },
    &Func {
        name: "dbTopEntries",
        doc: "",
    },
    &Func {
        name: "dbBottomEntries",
        doc: "",
    },
    &Func { name: "dbCount", doc: "" },
    &Func { name: "dbRank", doc: "" },
    &Func { name: "str", doc: "" },
    &Func { name: "toString", doc: "" },
    &Func { name: "toInt", doc: "" },
    &Func { name: "toInt64", doc: "" },
    &Func { name: "toFloat", doc: "" },
    &Func {
        name: "toDuration",
        doc: "",
    },
    &Func { name: "toRune", doc: "" },
    &Func { name: "toByte", doc: "" },
    &Func { name: "hasPrefix", doc: "" },
    &Func { name: "hasSuffix", doc: "" },
    &Func { name: "joinStr", doc: "" },
    &Func { name: "lower", doc: "" },
    &Func { name: "slice", doc: "" },
    &Func { name: "split", doc: "" },
    &Func { name: "title", doc: "" },
    &Func { name: "trimSpace", doc: "" },
    &Func { name: "upper", doc: "" },
    &Func { name: "urlescape", doc: "" },
    &Func {
        name: "urlunescape",
        doc: "",
    },
    &Func { name: "print", doc: "" },
    &Func { name: "println", doc: "" },
    &Func { name: "printf", doc: "" },
    &Func {
        name: "sanitizeText",
        doc: "",
    },
    &Func {
        name: "reQuoteMeta",
        doc: "",
    },
    &Func { name: "add", doc: "" },
    &Func { name: "cbrt", doc: "" },
    &Func { name: "div", doc: "" },
    &Func { name: "fdiv", doc: "" },
    &Func { name: "log", doc: "" },
    &Func { name: "mathConst", doc: "" },
    &Func { name: "max", doc: "" },
    &Func { name: "min", doc: "" },
    &Func { name: "mod", doc: "" },
    &Func { name: "mult", doc: "" },
    &Func { name: "pow", doc: "" },
    &Func { name: "round", doc: "" },
    &Func { name: "roundCeil", doc: "" },
    &Func { name: "roundEven", doc: "" },
    &Func {
        name: "roundFloor",
        doc: "",
    },
    &Func { name: "sqrt", doc: "" },
    &Func { name: "sub", doc: "" },
    &Func {
        name: "bitwiseAnd",
        doc: "",
    },
    &Func { name: "bitwiseOr", doc: "" },
    &Func {
        name: "bitwiseXor",
        doc: "",
    },
    &Func {
        name: "bitwiseNot",
        doc: "",
    },
    &Func {
        name: "bitwiseAndNot",
        doc: "",
    },
    &Func {
        name: "bitwiseLeftShift",
        doc: "",
    },
    &Func {
        name: "bitwiseRightShift",
        doc: "",
    },
    &Func {
        name: "humanizeThousands",
        doc: "",
    },
    &Func { name: "dict", doc: "" },
    &Func { name: "sdict", doc: "" },
    &Func {
        name: "structToSdict",
        doc: "",
    },
    &Func { name: "cembed", doc: "" },
    &Func { name: "cslice", doc: "" },
    &Func {
        name: "complexMessage",
        doc: "",
    },
    &Func {
        name: "complexMessageEdit",
        doc: "",
    },
    &Func { name: "kindOf", doc: "" },
    &Func { name: "adjective", doc: "" },
    &Func { name: "in", doc: "" },
    &Func { name: "inFold", doc: "" },
    &Func { name: "json", doc: "" },
    &Func {
        name: "jsonToSdict",
        doc: "",
    },
    &Func { name: "noun", doc: "" },
    &Func { name: "randInt", doc: "" },
    &Func { name: "roleAbove", doc: "" },
    &Func { name: "seq", doc: "" },
    &Func { name: "shuffle", doc: "" },
    &Func { name: "verb", doc: "" },
    &Func {
        name: "currentTime",
        doc: "",
    },
    &Func { name: "parseTime", doc: "" },
    &Func {
        name: "formatTime",
        doc: "",
    },
    &Func {
        name: "loadLocation",
        doc: "",
    },
    &Func { name: "newDate", doc: "" },
    &Func {
        name: "snowflakeToTime",
        doc: "",
    },
    &Func {
        name: "timestampToTime",
        doc: "",
    },
    &Func {
        name: "weekNumber",
        doc: "",
    },
    &Func {
        name: "humanizeDurationHours",
        doc: "",
    },
    &Func {
        name: "humanizeDurationMinutes",
        doc: "",
    },
    &Func {
        name: "humanizeDurationSeconds",
        doc: "",
    },
    &Func {
        name: "humanizeTimeSinceDays",
        doc: "",
    },
];
