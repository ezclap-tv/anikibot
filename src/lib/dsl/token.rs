use logos::Logos;

#[derive(Debug, PartialEq, Clone, Copy, Logos)]
enum Token {
    #[token(".")]
    Dot,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("=>")]
    FatArrow,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex("block|msg|tx|now|suicide|selfdestruct|addmod")]
    #[regex("mulmod|sha3|keccak256|log0|log1|log2|log3|log4")]
    #[regex("sha256|ecrecover|ripemd160|assert|revert|require")]
    IdentifierBuiltin,

    #[token("contract")]
    DeclarationContract,

    #[token("library")]
    DeclarationLibrary,

    #[token("interface")]
    DeclarationInterface,

    #[token("enum")]
    DeclarationEnum,

    #[token("struct")]
    DeclarationStruct,

    #[token("modifier")]
    DeclarationModifier,

    #[token("event")]
    DeclarationEvent,

    #[token("function")]
    DeclarationFunction,

    #[token("var")]
    DeclarationVar,

    #[token("anonymous")]
    KeywordAnonymous,

    #[token("as")]
    KeywordAs,

    #[token("assembly")]
    KeywordAssembly,

    #[token("break")]
    KeywordBreak,

    #[token("constant")]
    KeywordConstant,

    #[token("continue")]
    KeywordContinue,

    #[token("do")]
    KeywordDo,

    #[token("delete")]
    KeywordDelete,

    #[token("else")]
    KeywordElse,

    #[token("external")]
    KeywordExternal,

    #[token("for")]
    KeywordFor,

    // FIXME: Should able to handle hex literals on lexer-level!
    #[token("hex")]
    KeywordHex,

    #[token("if")]
    KeywordIf,

    #[token("indexed")]
    KeywordIndexed,

    #[token("internal")]
    KeywordInternal,

    #[token("import")]
    KeywordImport,

    #[token("is")]
    KeywordIs,

    #[token("mapping")]
    KeywordMapping,

    #[token("memory")]
    KeywordMemory,

    #[token("new")]
    KeywordNew,

    #[token("payable")]
    KeywordPayable,

    #[token("public")]
    KeywordPublic,

    #[token("pragma")]
    KeywordPragma,

    #[token("private")]
    KeywordPrivate,

    #[token("pure")]
    KeywordPure,

    #[token("return")]
    KeywordReturn,

    #[token("returns")]
    KeywordReturns,

    #[token("storage")]
    KeywordStorage,

    #[token("super")]
    KeywordSuper,

    #[token("this")]
    KeywordThis,

    #[token("throw")]
    KeywordThrow,

    #[token("using")]
    KeywordUsing,

    #[token("view")]
    KeywordView,

    #[token("while")]
    KeywordWhile,

    #[regex("abstract|after|case|catch|default|final|in")]
    #[regex("inline|let|match|null|of|relocatable|static")]
    #[regex("switch|try|type|typeof")]
    ReservedWord,

    #[token("bool")]
    TypeBool,

    #[token("address")]
    TypeAddress,

    #[token("string")]
    TypeString,

    #[regex("byte|bytes[1-2][0-9]?|bytes3[0-2]?|bytes[4-9]")]
    #[callback = "validate_bytes"]
    TypeByte,

    #[token("bytes")]
    TypeBytes,

    #[token("int")]
    #[callback = "default_size"]
    TypeInt,

    #[token("uint")]
    #[callback = "default_size"]
    TypeUint,

    #[regex("int(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144)")]
    #[regex("int(152|160|168|176|184|192|200|208|216|224|232|240|248|256)")]
    #[callback = "validate_int"]
    TypeIntN,

    #[regex("uint(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144)")]
    #[regex("uint(152|160|168|176|184|192|200|208|216|224|232|240|248|256)")]
    #[callback = "validate_uint"]
    TypeUintN,

    #[regex("fixed([1-9][0-9]?[0-9]?x[0-9][0-9]?)?")]
    #[callback = "validate_fixed"]
    TypeFixed,

    #[regex("ufixed([1-9][0-9]?[0-9]?x[0-9][0-9]?)?")]
    #[callback = "validate_fixed"]
    TypeUfixed,

    #[token("true")]
    LiteralTrue,

    #[token("false")]
    LiteralFalse,

    #[regex("0[xX][0-9a-fA-F]+")]
    LiteralHex,

    #[regex("[0-9]+")]
    LiteralInteger,

    #[regex("[0-9]*\\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+")]
    #[callback = "rational_to_integer"]
    LiteralRational,

    #[regex("\"([^\"\\\\]|\\\\.)*\"")]
    #[regex("'([^'\\\\]|\\\\.)*'")]
    LiteralString,

    #[token("ether")]
    UnitEther,

    #[token("finney")]
    UnitFinney,

    #[token("szabo")]
    UnitSzabo,

    #[token("wei")]
    UnitWei,

    #[token("years")]
    UnitTimeYears,

    #[token("weeks")]
    UnitTimeWeeks,

    #[token("days")]
    UnitTimeDays,

    #[token("hours")]
    UnitTimeHours,

    #[token("minutes")]
    UnitTimeMinutes,

    #[token("seconds")]
    UnitTimeSeconds,

    #[token(":=")]
    AssemblyBind,

    #[token("=:")]
    AssemblyAssign,

    #[token("++")]
    OperatorIncrement,

    #[token("--")]
    OperatorDecrement,

    #[token("!")]
    OperatorLogicalNot,

    #[token("~")]
    OperatorBitNot,

    #[token("*")]
    OperatorMultiplication,

    #[token("/")]
    OperatorDivision,

    #[token("%")]
    OperatorRemainder,

    #[token("**")]
    OperatorExponent,

    #[token("+")]
    OperatorAddition,

    #[token("-")]
    OperatorSubtraction,

    #[token("<<")]
    OperatorBitShiftLeft,

    #[token(">>")]
    OperatorBitShiftRight,

    #[token("<")]
    OperatorLesser,

    #[token("<=")]
    OperatorLesserEquals,

    #[token(">")]
    OperatorGreater,

    #[token(">=")]
    OperatorGreaterEquals,

    #[token("==")]
    OperatorEquality,

    #[token("!=")]
    OperatorInequality,

    #[token("&")]
    OperatorBitAnd,

    #[token("^")]
    OperatorBitXor,

    #[token("|")]
    OperatorBitOr,

    #[token("&&")]
    OperatorLogicalAnd,

    #[token("||")]
    OperatorLogicalOr,

    #[token("?")]
    OperatorConditional,

    #[token("=")]
    Assign,

    #[token("+=")]
    AssignAddition,

    #[token("-=")]
    AssignSubtraction,

    #[token("*=")]
    AssignMultiplication,

    #[token("/=")]
    AssignDivision,

    #[token("%=")]
    AssignRemainder,

    #[token("<<=")]
    AssignBitShiftLeft,

    #[token(">>=")]
    AssignBitShiftRight,

    #[token("&=")]
    AssignBitAnd,

    #[token("^=")]
    AssignBitXor,

    #[token("|=")]
    AssignBitOr,

    #[regex("//[^\n]*")]
    Comment,

    #[regex("/\*.*\*")]
    MultilineToken,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}
