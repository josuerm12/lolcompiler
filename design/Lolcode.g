grammar Lolcode;


lolcode      : HAI haiBlock KBYE EOF ;
haiBlock     : varDef? preHead ;
preHead      : comment preHead | head body | stmtNc body | ;
head         : MAEK_HEAD varDef? comment* (title comment*)? MKAY ;
title        : GIMMEH_TITLE varDef? inline OIC ;
body         : stmt* ;
stmt         : comment | stmtNc ;
stmtNc       : paragraph | bold | list | NEWLINE | link | varUse | TEXT ;
paragraph    : MAEK_PARAGRAF varDef? parStmt* MKAY ;
parStmt      : comment | bold | list | NEWLINE | link | varUse | TEXT ;
bold         : GIMMEH_BOLD varDef? inline OIC ;
list         : MAEK_LIST item (item | comment)* MKAY ;
item         : GIMMEH_ITEM varDef? (TEXT | varUse | bold)* OIC ;
inline       : (TEXT | varUse)* ;
comment      : OBTW TEXT* TLDR ;
link         : GIMMEH_LINX TEXT OIC ;
varDef       : IHAZ TEXT ITIZ TEXT+ MKAY ;
varUse       : GIMMEH_VAR TEXT OIC ;


HAI            : '#' H A I ;
KBYE           : '#' K B Y E ;
OBTW           : '#' O B T W ;
TLDR           : '#' T L D R ;
MAEK_HEAD      : '#' M A E K SP H E A D ;
MAEK_PARAGRAF  : '#' M A E K SP P A R A G R A F ;
MAEK_LIST      : '#' M A E K SP L I S T ;
MKAY           : '#' M K A Y ;
GIMMEH_TITLE   : '#' G I M M E H SP T I T L E ;
GIMMEH_BOLD    : '#' G I M M E H SP B O L D ;
GIMMEH_ITEM    : '#' G I M M E H SP I T E M ;
GIMMEH_LINX    : '#' G I M M E H SP L I N X ;
GIMMEH_VAR     : '#' G I M M E H SP V A R ;
OIC            : '#' O I C ;
IHAZ           : '#' I H A Z ;
ITIZ           : '#' I T I Z ;
NEWLINE        : '#' N E W L I N E ;


TEXT : ( 'a'..'z' | 'A'..'Z' | '0'..'9' | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/' )+ ;

WS : ( ' ' | '\t' | '\r' | '\n' )+ { $channel = HIDDEN; } ;

fragment SP : ( ' ' | '\t' | '\r' | '\n' )+ ;
fragment A : 'a' | 'A' ;
fragment B : 'b' | 'B' ;
fragment C : 'c' | 'C' ;
fragment D : 'd' | 'D' ;
fragment E : 'e' | 'E' ;
fragment F : 'f' | 'F' ;
fragment G : 'g' | 'G' ;
fragment H : 'h' | 'H' ;
fragment I : 'i' | 'I' ;
fragment J : 'j' | 'J' ;
fragment K : 'k' | 'K' ;
fragment L : 'l' | 'L' ;
fragment M : 'm' | 'M' ;
fragment N : 'n' | 'N' ;
fragment O : 'o' | 'O' ;
fragment P : 'p' | 'P' ;
fragment Q : 'q' | 'Q' ;
fragment R : 'r' | 'R' ;
fragment S : 's' | 'S' ;
fragment T : 't' | 'T' ;
fragment U : 'u' | 'U' ;
fragment V : 'v' | 'V' ;
fragment W : 'w' | 'W' ;
fragment X : 'x' | 'X' ;
fragment Y : 'y' | 'Y' ;
fragment Z : 'z' | 'Z' ;
